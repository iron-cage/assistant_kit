# Invariant: Zero Dependencies

### Scope

- **Purpose**: Keep `claude_terminal_core` at Layer `*` — no workspace dependencies, no external runtime crates — so it can be depended on from anywhere in the workspace without creating an ordering constraint.
- **Governs**: `Cargo.toml`'s `[dependencies]` section.
- **In Scope**: Runtime dependencies of any kind.
- **Out of Scope**: `[dev-dependencies]`; those do not appear in the built library.

### Rule

`[dependencies]` MUST remain empty. The crate builds against `core`/`std` only. Its entire input is a `&str` and its entire output is a `String`.

**Rationale — Layer \*.** The workspace enforces a layer ordering in `module/assistant/tests/workspace_invariants.rs`: CL-1 requires that a Layer `*` crate have zero workspace dependencies, and in exchange CL-2 exempts *depending on* a Layer `*` crate from the downward-flow check. Staying at Layer `*` is what lets both `claude_runner` (Layer 2) and any future consumer use this crate without an argument about where rendering belongs in the ordering.

**Rationale — why not an emulator crate.** `vte`, `termwiz` and friends are terminal *emulators*: they carry a screen model — grid, scrollback, scroll regions, alternate screen. This crate exists specifically not to have one ([002_line_renderer_boundary.md](002_line_renderer_boundary.md)). Adopting one would not simplify the scanner; it would replace a documented, testable boundary with a full emulation whose output for a repainting program is a *screen snapshot* rather than a transcript — the opposite of what the consumer needs. The dependency would be paid in order to get behaviour that must then be worked around.

**Rationale — why this is not folded into `claude_pty_core`.** That crate already carries hand-rolled POSIX FFI and a scoped `unsafe` exception. Interpreting escape sequences needs neither. Folding this in would mean a caller holding captured bytes — with no pty in sight — links `ffi.rs` and its `extern "C"` declarations to call a pure string function. The two crates split along a real boundary: `claude_pty_core` owns the device, this one owns the protocol spoken over it.

### Verification

```bash
cd module/claude_terminal_core && cargo tree --edges normal --depth 1
```

Shows `claude_terminal_core v0.1.0` with no children. The workspace-level check is `cl1_no_same_layer_deps` in `module/assistant/tests/workspace_invariants.rs`, which fails if this crate acquires any workspace dependency.

A stricter local check — that the manifest has no dependency lines at all:

```bash
cd module/claude_terminal_core && awk '/^\[dependencies\]/{d=1;next} /^\[/{d=0} d && /=/{print}' Cargo.toml
```

Prints nothing when the invariant holds.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `Cargo.toml` | The empty `[dependencies]` section and its comment |
| doc | [002_line_renderer_boundary.md](002_line_renderer_boundary.md) | Why no emulator crate is wanted |
| doc | [`claude_pty_core` invariant/002](../../../claude_pty_core/docs/invariant/002_zero_dependencies.md) | The sibling crate's matching guarantee |
| test | `../../../assistant/tests/workspace_invariants.rs` | CL-1 Layer `*` zero-dependency check |

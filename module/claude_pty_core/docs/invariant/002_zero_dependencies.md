# Invariant: Zero Dependencies

### Scope

- **Purpose**: Keep `claude_pty_core` at Layer `*` — no workspace dependencies, no external runtime crates — so it can be depended on from anywhere in the workspace without creating an ordering constraint.
- **Governs**: `Cargo.toml`'s `[dependencies]` section.
- **In Scope**: Runtime dependencies of any kind.
- **Out of Scope**: `[dev-dependencies]`, which may use `tempfile` and other test-only crates; those do not appear in the built library.

### Rule

`[dependencies]` MUST remain empty. The crate builds against `std` and the platform's libc symbols only, the latter reached through `extern "C"` declarations rather than through the `libc` crate.

**Rationale — Layer \*.** The workspace enforces a layer ordering in `module/assistant/tests/workspace_invariants.rs`: CL-1 requires that a Layer `*` crate have zero workspace dependencies, and in exchange CL-2 exempts *depending on* a Layer `*` crate from the downward-flow check. Staying at Layer `*` is what lets `claude_daemon_core` (Layer 1) use this crate without argument about which layer terminal mechanics belong in.

**Rationale — why not `libc`.** Adding `libc` would trade six `extern "C"` lines for a dependency, and buy nothing this crate uses: the six symbols it needs (`posix_openpt`, `grantpt`, `unlockpt`, `ptsname_r`, `ioctl`, `setsid`) have stable, unchanging signatures. The ioctl request numbers (`TIOCSWINSZ`, `TIOCSCTTY`) are Linux-specific constants either way.

**Rationale — why not a PTY crate.** Both plausible candidates were evaluated and rejected on hard constraints, not preference:

| Crate | Blocker |
|-------|---------|
| `pty-process` | `edition = "2024"`, requiring Rust 1.85; the workspace pins `rust-version = "1.75"` |
| `portable-pty` | `anyhow` in its public API, against a workspace that mandates one error convention; 18 transitive dependencies |

The constraint this invariant records is that reintroducing either would break something concrete — the MSRV floor or the error-handling rule — not merely add weight.

### Verification

```bash
cd module/claude_pty_core && cargo tree --edges normal --depth 1
```

Shows `claude_pty_core v0.1.0` with no children. The workspace-level check is `cl1_no_same_layer_deps` in `module/assistant/tests/workspace_invariants.rs`, which fails if this crate acquires any workspace dependency.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `Cargo.toml` | The empty `[dependencies]` section and its comment |
| source | `src/ffi.rs` | The `extern "C"` declarations that replace `libc` |
| doc | [001_unsafe_containment.md](001_unsafe_containment.md) | How the cost of hand-rolled FFI is bounded |
| test | `../../assistant/tests/workspace_invariants.rs` | CL-1 Layer `*` zero-dependency check |

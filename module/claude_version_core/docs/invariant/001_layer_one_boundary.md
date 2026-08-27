# Invariant: Layer 1 Boundary

### Scope

- **Purpose**: Keep `claude_version_core` a Layer 1 domain crate — depending only on Layer 0, and never naming a CLI framework type.
- **Responsibility**: State the dependency and error-type constraints, define the enforcement commands, document violation consequences, and record where this crate's own doc references actually resolve.
- **In Scope**: Dependency restriction (INV-1), no CLI framework (INV-2), error-type boundary (INV-3), documentation lints (INV-4).
- **Out of Scope**: Cross-file literal consistency (→ `002_alias_literal_consistency.md`), API contracts (→ `api/`).

### Invariant Statement

| ID | Invariant |
|----|-----------|
| INV-1 | `claude_version_core` depends on exactly one workspace crate: `claude_core` (Layer 0) |
| INV-2 | Neither `unilang`, `error_tools`, `anyhow`, nor `thiserror` appears in the manifest |
| INV-3 | Every fallible public function returns `CoreError`; `ErrorData` is never named except in doc comments describing the Layer 2 adaptation |
| INV-4 | `#![warn(missing_docs)]` and `#![warn(missing_debug_implementations)]` stay enabled in `lib.rs` |

### Enforcement Mechanism

**INV-1** — the dependency set is exactly `claude_core`:

```bash
cargo tree -p claude_version_core --edges normal --depth 1
# Expected: claude_version_core, with claude_core as its only dependency line
```

**INV-2** — no CLI framework or error-crate dependency:

```bash
grep -nE 'unilang|error_tools|anyhow|thiserror' module/claude_version_core/Cargo.toml
# Expected: empty output
```

**INV-3** — `ErrorData` and `unilang` appear only inside doc comments:

```bash
grep -rnE 'ErrorData|unilang' module/claude_version_core/src/
# Expected: only `//!` and `///` lines in src/lib.rs describing the Layer 2
#   adaptation pattern. No code line may match.
```

**INV-4** — both lints remain declared:

```bash
grep -n 'warn( missing_docs )\|warn( missing_debug_implementations )' module/claude_version_core/src/lib.rs
# Expected: two lines
```

### Violation Consequences

- **INV-1 violated:** A dependency on any Layer 2 crate inverts the layering — `claude_version`
  depends on this crate, so the edge would close a cycle. A dependency on another Layer 1 crate
  couples two domains that are meant to be independently consumable through `dream`.
- **INV-2 violated:** `unilang` is the CLI framework. Linking it here would pull argument
  parsing and command dispatch into every library consumer of `dream::version`, which is the
  precise cost the Layer 1 / Layer 2 split exists to avoid. `error_tools`, `anyhow`, and
  `thiserror` are excluded under workspace `docs/invariant/005_dependency_management.md`.
- **INV-3 violated:** `ErrorData` is a `unilang` type; naming it in a signature makes INV-2
  unsatisfiable. The adaptation belongs at the Layer 2 call site —
  `.map_err( |e| ErrorData::new( code, e.to_string() ) )`.
- **INV-4 violated:** `missing_docs` is what keeps the `api/` instances writable — an
  undocumented public item cannot be contract-documented from the source alone.

### Known Documentation Split

Doc comments in `src/` cite four bare `docs/…` paths. Two now resolve inside this crate; two
still resolve only in the Layer 2 sibling. The remaining two are a genuine leaf-proximity
deviation — the behavior is owned here, the specification lives one layer up — recorded so a
reader following a reference is not left searching:

| Cited from `src/` | Resolves | At |
|-------------------|----------|----|
| `docs/algorithm/002_config_resolution.md` | in-crate | `../algorithm/002_config_resolution.md` |
| `docs/pattern/002_parameter_trace.md` | in-crate | `../pattern/002_parameter_trace.md` |
| `docs/feature/001_version_management.md` | Layer 2 only | `../../../claude_version/docs/feature/001_version_management.md` |
| `docs/feature/007_params_command.md` | Layer 2 only | `../../../claude_version/docs/feature/007_params_command.md` |

The two in-crate rows are listed rather than dropped because a same-named instance also exists
at Layer 2 — `claude_version/docs/algorithm/002_config_resolution.md` and
`.../pattern/002_parameter_trace.md`. That is a deliberate two-layer split, not a duplicate:
the instance here documents what this crate implements, the Layer 2 instance documents how the
CLI command consumes and renders it. A bare citation in `src/` means the local one.

Enumerate the citing sites with:

```bash
grep -rnoE 'docs/[a-z_]+/[0-9]{3}_[a-z_]+\.md' module/claude_version_core/src/
```

Adding a new bare `docs/…` citation to `src/` without a matching row above compounds the
deviation. Either write the doc in this crate, or cite the Layer 2 path relatively.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [002_alias_literal_consistency.md](002_alias_literal_consistency.md) | The other cross-file constraint this crate carries |
| doc | [api/001_core_surface.md](../api/001_core_surface.md) | The `CoreError` contract INV-3 protects |
| doc | workspace `docs/invariant/005_dependency_management.md` | Workspace-wide rule excluding `anyhow` and `thiserror` |
| source | `../../Cargo.toml` | Dep declarations that must satisfy INV-1 and INV-2 |
| source | `../../src/lib.rs` | `CoreError` definition and the lint declarations |

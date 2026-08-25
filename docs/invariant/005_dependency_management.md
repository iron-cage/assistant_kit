# Invariant: Dependency Management

### Scope

- **Purpose**: Document the workspace dependency management policy governing external dep declaration, centralization, and publish readiness.
- **Responsibility**: State the workspace-centralization rule, the `workspace = true` enforcement, the out-of-workspace path dep version requirement, publish metadata requirements, and the error-handling dependency exclusivity rule.
- **In Scope**: Workspace dep centralization, `workspace = true` usage, out-of-workspace path dep version field, publish metadata for crates.io, error-handling dependency exclusivity (`error_tools` only).
- **Out of Scope**: Privacy invariant (→ `invariant/001_privacy_invariant.md`), versioning strategy (→ `invariant/002_versioning_strategy.md`).

### Invariant Statement

All external dependencies are declared once in `[workspace.dependencies]` and consumed via `{ workspace = true }` in individual crates.

**Rule 1 — Centralization:** Every external dep (crates.io packages) used by any workspace member must have an entry in `[workspace.dependencies]` in the root `Cargo.toml`. No crate-level inline version declarations for shared external deps.

**Rule 2 — Workspace reference:** Every crate-level `Cargo.toml` that uses an external dep must reference it as `dep_name = { workspace = true }` (plus any crate-specific `optional = true` or `features = [...]` overrides). A bare `dep_name = "x.y.z"` in a crate `Cargo.toml` is a violation.

**Rule 3 — Out-of-workspace path deps must carry a `version` field:**
```toml
# Correct — publishable because version resolves to crates.io on `cargo publish`
[workspace.dependencies.cli_fmt]
version = "^0.8"
path = "../../wtools/dev/module/core/cli_fmt"

# Violation — cargo publish rejects path-only deps
[workspace.dependencies.cli_fmt]
path = "../../wtools/dev/module/core/cli_fmt"
```
Cargo uses `path` for local builds and `version` for publishing. A `path`-only dep makes every crate in the dependency chain unpublishable.

**Rule 4 — Publish metadata for publishable crates:** Any workspace member without `publish = false` must carry the following fields before it can be published to crates.io:
- `license` or `license-file`
- `description`
- `repository`
- `authors`
- `rust-version`
- `keywords` and `categories` (recommended; required for discoverability)

**Rule 5 — Error handling depends on `error_tools` alone:** No workspace member declares `thiserror` or `anyhow` as a direct dependency. `error_tools` is the single error-handling dependency; it is a facade over both (`error_tools::typed` for typed errors, `error_tools::untyped` for untyped).

Crates that need a typed error define it by hand — a plain `enum` with manual `Display` and `Error` impls — rather than deriving it. This is deliberate, not an oversight:

> **Decision (2026-08-25):** routing the hand-rolled error enums through `error_tools::typed`'s `#[ derive( Error ) ]` was considered and rejected. `thiserror` 2's derive macros expand to absolute `::thiserror` paths, so every crate deriving `Error` would need `thiserror` as its own direct dependency — `error_tools`' `src/error/typed.rs` states plainly that a `use error_tools::dependency::thiserror;` alias "is no longer sufficient". Ten hand-rolled enums across nine crates keep their manual impls so that this rule stays true. The cost is some boilerplate; the benefit is that the workspace has exactly one error-handling dependency rather than two idioms in the same tree.

### Enforcement Mechanism

**Centralization check:** Search for bare version declarations in crate `Cargo.toml` files:
```bash
grep -rn '^\w* = "' module/*/Cargo.toml contract/*/Cargo.toml
```
Any match that is not in `[workspace.dependencies]` is a violation.

**Workspace reference check:** All external deps in crate files must include `workspace = true`:
```bash
grep -rn 'version = "' module/*/Cargo.toml | grep -v 'workspace.package'
```

**Out-of-workspace path dep check:** All `path = "../..` entries in `[workspace.dependencies]` must also have a `version` field. Verify before any publish attempt:
```bash
grep -A3 'path = "\.\.' Cargo.toml | grep -v 'version'
```

**Error-handling exclusivity check:** No crate may name `thiserror` or `anyhow` directly:
```bash
grep -lE '^\s*(thiserror|anyhow)\b|^\s*\[[^]]*dependencies\.(thiserror|anyhow)\]' module/*/Cargo.toml
```
Both declaration forms must be covered — the inline `thiserror = { ... }` / `thiserror.workspace = true` shape and the `[dependencies.thiserror]` section header shape. A pattern matching only the first silently passes a manifest using the second. Empty output means the invariant holds. Any listed file is a violation — the fix is to consume the dependency through `error_tools`, not to add the direct dep.

**Version freshness:** External dep versions in `[workspace.dependencies]` must track latest stable published releases. Companion crates (`error_tools`, `unilang`, `data_fmt`, `cli_fmt`, `test_tools`) share an author with this workspace — update promptly when new minor versions are published. Third-party crates (`ureq`, `tempfile`, etc.) follow semver: major bumps require API migration assessment; minor/patch bumps are routine.

### Violation Consequences

- A bare version in a crate `Cargo.toml` creates invisible version divergence that silently overrides the workspace-managed version
- A path-only dep without `version` causes `cargo publish` to fail with "dependency … must be specified using a version" for every crate that depends on it
- Missing publish metadata causes crates.io to reject the publish with "field … is required"
- Stale dep versions expose the workspace to resolved security vulnerabilities and miss upstream bug fixes
- A direct `thiserror` or `anyhow` dep splits error handling across two idioms and forfeits the single-facade guarantee `error_tools` exists to provide

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_privacy_invariant.md](001_privacy_invariant.md) | Forbidden dep types (private workspace path deps) |
| [invariant/002_versioning_strategy.md](002_versioning_strategy.md) | Shared version policy for workspace members |

### Sources

| File | Relationship |
|------|--------------|
| `../../Cargo.toml` | Workspace dependency declarations |

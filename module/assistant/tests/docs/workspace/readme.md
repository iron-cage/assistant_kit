# workspace/

Test surface specifications for workspace-level doc instances (`docs/` at workspace root). Tests verify structural invariants across all workspace crates using static analysis of Cargo.toml files and dependency graphs.

### Scope

- **Purpose**: Mirror the statically-verifiable subset of workspace-level doc instances as test specs.
- **Responsibility**: Index the three root `docs/` categories whose instances can be asserted from Cargo.toml manifests and the on-disk doc tree.
- **In Scope**: `feature/`, `invariant/`, and `pattern/` — backed by `tests/workspace_invariants.rs` (manifest and dependency-graph assertions) and `tests/entity_consistency.rs` (registry index counts).
- **Out of Scope**: The root `docs/` categories with no statically-assertable surface — [`error/`](../../../../../docs/error/readme.md) catalogs Claude Code's own external error messages, and [`integration/`](../../../../../docs/integration/readme.md) specifies a protocol for consumer workspaces outside this repository. Neither is a property of this workspace's manifests or doc tree, so neither is mirrored here.

### Responsibility Table

| Name | Responsibility |
|------|----------------|
| `feature/` | Test specs for workspace feature doc instances |
| `invariant/` | Test specs for workspace invariant doc instances |
| `pattern/` | Test specs for workspace pattern doc instances |

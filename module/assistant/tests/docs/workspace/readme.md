# workspace/

Test surface specifications for workspace-level doc instances (`docs/` at workspace root). Tests verify structural invariants across all workspace crates using static analysis of Cargo.toml files and dependency graphs.

### Responsibility Table

| Name | Responsibility |
|------|----------------|
| `feature/` | Test specs for workspace feature doc instances |
| `invariant/` | Test specs for workspace invariant doc instances |
| `pattern/` | Test specs for workspace pattern doc instances |

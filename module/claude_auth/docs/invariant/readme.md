# Invariant Doc Entity

### Scope

- **Purpose**: Document structural constraints that `claude_auth` must always satisfy.
- **Responsibility**: Index of invariant doc instances covering the dependency shape and the offline parse core.
- **In Scope**: Workspace-dep exclusion, optional third-party dep, feature-gate placement, offline testability.
- **Out of Scope**: Wire-protocol behavior (→ `feature/`), signature contracts (→ `api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Zero Workspace Dependencies](001_zero_workspace_deps.md) | Layer `*` standalone: no workspace deps, `ureq` optional only | ✅ |
| 002 | [Offline Parse Core](002_offline_parse_core.md) | Everything but the network call stays unconditional and pure | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating invariant doc instances | ✅ |

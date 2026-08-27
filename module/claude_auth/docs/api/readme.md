# API Doc Entity

### Scope

- **Purpose**: Pin the public library contract of `claude_auth` so consumers can depend on it without reading the source.
- **Responsibility**: Index of API doc instances covering signatures, error contracts, and availability by feature.
- **In Scope**: Public constants, types, and functions exported from `lib.rs`.
- **Out of Scope**: Behavioral rationale (→ `feature/`), structural constraints (→ `invariant/`), private helpers.

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Auth Surface](001_auth_surface.md) | Every exported constant, type, and function with its contract | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating API doc instances | ✅ |

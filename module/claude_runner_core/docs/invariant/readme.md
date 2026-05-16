# Invariant Doc Entity

### Scope

- **Purpose**: Document non-functional constraints that the library must always satisfy.
- **Responsibility**: Index of invariant doc instances covering the single-execution-point rule and NFR conformance.
- **In Scope**: Execution centralization invariant, dependency count constraints, type-safety requirements.
- **Out of Scope**: Feature behavior (→ `feature/`), builder API (→ `api/`, `pattern/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Single Execution Point](001_single_execution_point.md) | All claude spawning goes through one location | ✅ |
| 002 | [NFR Conformance](002_nfr_conformance.md) | Dependency count, performance, and type-safety constraints | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating invariant doc instances | ✅ |

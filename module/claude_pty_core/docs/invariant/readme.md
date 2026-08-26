# Invariant Doc Entity

### Scope

- **Purpose**: Record the constraints that must hold for every build of `claude_pty_core`, each with a mechanical check that fails the test suite when it stops holding.
- **Responsibility**: Index of invariant doc instances covering unsafe-code containment and the crate's zero-dependency guarantee.
- **In Scope**: Which modules may contain `unsafe`; what the crate is permitted to depend on.
- **Out of Scope**: Behavioral capabilities (→ `feature/`), signature-level contracts (→ `api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Unsafe Containment](001_unsafe_containment.md) | All `unsafe` confined to `src/ffi.rs` | ✅ |
| 002 | [Zero Dependencies](002_zero_dependencies.md) | No runtime dependencies, workspace or external | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating invariant doc instances | ✅ |

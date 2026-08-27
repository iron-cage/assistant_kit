# Invariant Doc Entity

### Scope

- **Purpose**: Record the constraints that must hold for every build of `claude_terminal_core`, each with a mechanical check that fails the test suite when it stops holding.
- **Responsibility**: Index of invariant doc instances covering the crate's zero-dependency guarantee and the renderer's modelling boundary.
- **In Scope**: What the crate is permitted to depend on; which control sequences may change the output text.
- **Out of Scope**: Behavioral capabilities (→ `feature/`), signature-level contracts (→ `api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Zero Dependencies](001_zero_dependencies.md) | No runtime dependencies, workspace or external | ✅ |
| 002 | [Line Renderer Boundary](002_line_renderer_boundary.md) | One line of cursor state, never a screen | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating invariant doc instances | ✅ |

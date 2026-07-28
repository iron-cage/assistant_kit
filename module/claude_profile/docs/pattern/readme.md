# Pattern Doc Entity

### Scope

- **Purpose**: Canonical reference for reusable design solutions applied across `claude_profile` — recurring problems with a documented, repeatable solution shape.
- **Responsibility**: Each instance documents one pattern: the problem it solves, the solution, where it applies, and its trade-offs. Command and feature docs reference these instances rather than re-deriving the reasoning per application site.
- **In Scope**: Cross-cutting design solutions applied (or planned to be applied) at more than one call site.
- **Out of Scope**: Single-site behavior with no reuse intent (→ `feature/`); algorithmic decision logic (→ `algorithm/`); measurable constraints (→ `invariant/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| — | [procedure](procedure.md) | Workflow for maintaining pattern instances | ✅ |
| 001 | [Grouped Column-Aligned Help Rendering](001_grouped_help_rendering.md) | Presentation scheme for grouped, `::`-aligned `.help` output on high-param-count commands | ✅ |

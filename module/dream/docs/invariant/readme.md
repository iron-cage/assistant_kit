# Invariant Doc Entity

### Scope

- **Purpose**: Document structural constraints that `dream` must always satisfy.
- **Responsibility**: Index of invariant doc instances covering the zero-own-logic constraint.
- **In Scope**: Zero-own-logic rule, re-export purity, layer dependency restrictions.
- **Out of Scope**: Feature-gate behavior (→ `feature/`), workspace privacy invariant (→ workspace `docs/invariant/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Zero Own Logic](001_no_own_logic.md) | No own types, functions, or traits defined in `src/` | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating invariant doc instances | ✅ |

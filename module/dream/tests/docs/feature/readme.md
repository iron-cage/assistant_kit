# Feature Doc Entity

### Scope

- **Purpose**: Document smoke-test coverage for the `dream` facade crate's feature-gate behavior.
- **Responsibility**: Index of feature test lenses covering the aggregation smoke tests.
- **In Scope**: Per-feature compile-check tests, type-access verification, feature isolation.
- **Out of Scope**: Zero-own-logic constraint (→ `invariant/`), workspace layering.

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Facade Aggregation](001_aggregation.md) | Smoke-test coverage for FR-1 through FR-10 | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating feature doc instances | ✅ |

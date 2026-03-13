# Invariant Doc Entity

### Scope

- **Purpose**: Document non-functional constraints that the agent_kit workspace must always satisfy.
- **Responsibility**: Index of invariant doc instances covering privacy, versioning, testing, and performance constraints.
- **In Scope**: Privacy invariant, shared versioning strategy, TDD baseline rule, performance expectations.
- **Out of Scope**: Feature behavior (→ `feature/`), crate layering pattern (→ `pattern/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Privacy Invariant](001_privacy_invariant.md) | Zero willbe knowledge rule and dependency flow | ✅ |
| 002 | [Versioning Strategy](002_versioning_strategy.md) | Shared workspace version and divergence policy | ✅ |
| 003 | [Testing Strategy](003_testing_strategy.md) | TDD baseline rule, test categories, baseline enforcement | ✅ |
| 004 | [Performance](004_performance.md) | Status verbosity modes, min_entries cost, count_entries cost model | ✅ |

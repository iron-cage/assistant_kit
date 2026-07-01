# Invariant Doc Entity

### Scope

- **Purpose**: Document non-functional constraints that the assistant workspace must always satisfy.
- **Responsibility**: Index of invariant doc instances covering privacy, versioning, testing, and performance constraints.
- **In Scope**: Privacy invariant, shared versioning strategy, TDD baseline rule, performance expectations, dependency management policy, doc entity index consistency.
- **Out of Scope**: Feature behavior (→ `feature/`), crate layering pattern (→ `pattern/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Privacy Invariant](001_privacy_invariant.md) | Privacy constraint: no private consumer workspace deps | ✅ |
| 002 | [Versioning Strategy](002_versioning_strategy.md) | Shared workspace version and divergence policy | ✅ |
| 003 | [Testing Strategy](003_testing_strategy.md) | TDD baseline rule, test categories, baseline enforcement | ✅ |
| 004 | [Performance](004_performance.md) | Status verbosity modes, min_entries cost, count_entries cost model | ✅ |
| 005 | [Dependency Management](005_dependency_management.md) | Workspace dep centralization, publish readiness, version freshness | ✅ |
| 006 | [Doc Entity Index Consistency](006_doc_entity_index_consistency.md) | entity.md count accuracy, file existence, NNN naming | ⏳ |

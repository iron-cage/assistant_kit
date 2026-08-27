# Invariant Doc Entity

Test specs for workspace-level invariant doc instances.

### Scope

- **Purpose**: Document test case planning for the workspace invariant doc instances that can be asserted statically.
- **Responsibility**: Index of per-invariant test spec files backed by `tests/workspace_invariants.rs` and `tests/entity_consistency.rs`.
- **In Scope**: Invariants 001, 002, 004, 005, and 006.
- **Out of Scope**: [`003_testing_strategy.md`](../../../../../../docs/invariant/003_testing_strategy.md) — a process invariant (TDD baseline, Level 3 enforcement) describing how testing is conducted, with no manifest or on-disk property for a test to assert; it is enforced by the `verb/` verification levels themselves, not by a spec here.

### Responsibility Table

| Name | Responsibility |
|------|----------------|
| [001_privacy_invariant.md](001_privacy_invariant.md) | Verify no forbidden dependencies leak private workspace knowledge |
| [002_versioning_strategy.md](002_versioning_strategy.md) | Verify workspace version inheritance and override consistency |
| [004_performance.md](004_performance.md) | Verify fast-path ops use no JSONL reads; count_entries uses byte search |
| [005_dependency_management.md](005_dependency_management.md) | Verify dependency centralization and publish readiness |
| [006_doc_entity_index_consistency.md](006_doc_entity_index_consistency.md) | Verify entity.md index counts match actual doc instances on disk |

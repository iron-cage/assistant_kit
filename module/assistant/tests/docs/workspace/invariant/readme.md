# invariant/

Test specs for workspace-level invariant doc instances.

### Responsibility Table

| Name | Responsibility |
|------|----------------|
| `001_privacy_invariant.md` | Verify no forbidden dependencies leak private workspace knowledge |
| `002_versioning_strategy.md` | Verify workspace version inheritance and override consistency |
| `004_performance.md` | Verify fast-path ops use no JSONL reads; count_entries uses byte search |
| `005_dependency_management.md` | Verify dependency centralization and publish readiness |
| `006_doc_entity_index_consistency.md` | Verify entity.md index counts match actual doc instances on disk |

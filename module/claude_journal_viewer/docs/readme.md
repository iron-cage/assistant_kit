# docs/

### Scope

**Responsibilities:** Feature requirements, CLI reference, and structural constraints for the `claude_journal_viewer` crate (`clj` binary).
**In Scope:** Feature requirements (`feature/`), CLI documentation (`cli/`), invariant constraints (`invariant/`), cross-entity index, doc graph.
**Out of Scope:** Source code (-> `src/`), automated tests (-> `tests/`), journal library internals (-> `claude_journal/docs/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `feature/` | Viewer design: CLI viewing, web viewing, query filtering |
| `invariant/` | Structural constraints: read-only access, web security |
| `cli/` | CLI reference: commands, params, types, groups, stories |
| `002_entity.md` | Cross-entity index: Master Doc Entities and Instances tables |
| `doc_graph.yml` | Cross-reference graph for all doc instances |

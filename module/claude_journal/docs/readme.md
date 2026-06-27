# docs/

### Scope

**Responsibilities:** Behavioral requirements, API contracts, and structural constraints for the `claude_journal` crate.
**In Scope:** Feature requirements (`feature/`), invariant constraints (`invariant/`), public API contracts (`api/`), cross-entity index, and doc graph.
**Out of Scope:** Source code (-> `src/`), automated tests (-> `tests/`), CLI documentation (-> `claude_journal_viewer/docs/cli/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `feature/` | Event journaling design: append-only logging, schema, rotation |
| `invariant/` | Structural constraints: append-only, crash safety, schema versioning |
| `api/` | Public library API contracts: JournalWriter, JournalReader, EventType |
| `002_entity.md` | Cross-entity index: Master Doc Entities and Instances tables |
| `doc_graph.yml` | Cross-reference graph for all doc instances |

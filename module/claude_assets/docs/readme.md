# docs/

### Scope

**Responsibilities:** Documentation for the `claude_assets` crate covering behavioral requirements, CLI command design, and non-functional constraints.
**In Scope:** Feature requirements (`feature/`), source-root and env constraints (`invariant/`), and doc cross-reference graph.
**Out of Scope:** Source code (→ `src/`), automated tests (→ `tests/`), build scripts (→ `verb/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `feature/` | Asset CLI command design and adapter behavior |
| `invariant/` | Source root resolution constraint and env var requirement |
| `doc_graph.yml` | Cross-reference graph for all doc instances |

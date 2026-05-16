# docs/

### Scope

**Responsibilities:** Documentation for the `claude_assets_core` crate covering behavioral requirements, design patterns, and non-functional constraints.
**In Scope:** Feature requirements (`feature/`), symlink-only install constraints (`invariant/`), and doc cross-reference graph.
**Out of Scope:** Source code (→ `src/`), automated tests (→ `tests/`), build scripts (→ `verb/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `feature/` | Artifact installer design and installation semantics |
| `invariant/` | Symlink-only install constraint and enforcement rules |
| `doc_graph.yml` | Cross-reference graph for all doc instances |

# docs/

### Scope

**Responsibilities:** Crate-level documentation for `dream`: behavioral requirements and structural invariants.
**In Scope:** Feature specifications (`feature/`), structural invariant constraints (`invariant/`), and doc cross-reference graph.
**Out of Scope:** Source code (→ `src/`), automated tests (→ `tests/`), build scripts (→ `verb/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `doc_graph.yml` | Cross-reference graph for all doc instances |
| `feature/` | Feature specifications: facade aggregation requirements (FR-1–FR-10) |
| `invariant/` | Invariant specifications: zero-own-logic structural constraint |

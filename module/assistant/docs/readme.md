# docs/

### Scope

**Responsibilities:** Documentation for the `assistant` crate covering behavioral requirements, aggregation design, and non-functional constraints for the `ast` super-app binary.
**In Scope:** Feature requirements (`feature/`), aggregation invariants (`invariant/`), and doc cross-reference graph.
**Out of Scope:** Source code (→ `src/`), automated tests (→ `tests/`), build scripts (→ `verb/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `feature/` | Super-app aggregation design and Layer 2 command registration |
| `invariant/` | Aggregation completeness constraint for Layer 2 crates |
| `doc_graph.yml` | Cross-reference graph for all doc instances |

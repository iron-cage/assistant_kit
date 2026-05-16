# docs/

### Scope

**Responsibilities:** Documentation for the `claude_runner_core` crate covering behavioral requirements, design patterns, API contracts, and non-functional constraints.
**In Scope:** Parameter reference (`claude_params/`), builder pattern design (`pattern/`), execution API contracts (`api/`), type definitions (`data_structure/`), feature requirements (`feature/`), execution constraints (`invariant/`), and doc cross-reference graph.
**Out of Scope:** Source code (→ `src/`), automated tests (→ `tests/`), build scripts (→ `verb/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `claude_params/` | Comprehensive reference for all claude binary parameters |
| `pattern/` | Builder pattern design and rationale |
| `api/` | Execution API contracts and method signatures |
| `data_structure/` | Type-safe configuration enum definitions |
| `feature/` | Execution control, dry-run, describe, and isolated subprocess features |
| `invariant/` | Single execution point and NFR conformance constraints |
| `doc_graph.yml` | Cross-reference graph for all doc instances |

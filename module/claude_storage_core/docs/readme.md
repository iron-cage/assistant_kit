# docs/

### Scope

**Responsibilities:** Documentation for the `claude_storage_core` library covering behavioral requirements, data models, algorithms, and API contracts.
**In Scope:** Feature requirements (`feature/`), in-memory data structures (`data_structure/`), computational algorithms (`algorithm/`), non-functional constraints (`invariant/`), public API contracts (`api/`), and doc cross-reference graph.
**Out of Scope:** Source code (→ `src/`), automated tests (→ `tests/`), build scripts (→ `verb/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `feature/` | User-facing capabilities and functional design |
| `data_structure/` | In-memory data models and their relationships |
| `algorithm/` | Computational procedures and their design rationale |
| `invariant/` | Non-functional constraints with measurable thresholds |
| `api/` | Public library interface and stability guarantees |
| `entity.md` | Doc Entity index for this crate's documentation scope |
| `doc_graph.yml` | Cross-reference graph for navigability analysis |

### Related Crates

| Crate | Relationship |
|-------|--------------|
| `claude_runner` | Depends on `algorithm/001_path_encoding.md` (Df() path encoding) via `scope_for()` and `to_storage_path_for()`; see `claude_runner/docs/algorithm/001_path_encoding.md` for its consumption-side documentation |

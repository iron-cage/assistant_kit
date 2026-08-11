# docs/

### Scope

**Responsibilities:** Documentation for the `claude_patch` crate covering CLI command design for both the `.patch.*` and `.param.*` subjects, and non-functional constraints.
**In Scope:** Feature requirements (`feature/`), read-only `.param.*` constraint (`invariant/`), and doc cross-reference graph.
**Out of Scope:** Source code (→ `src/`), automated tests (→ `tests/`), patch-component domain semantics (→ `claude_patch_core/docs/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `feature/` | `.patch.*` and `.param.*` CLI command design |
| `invariant/` | Read-only `.param.*` constraint and enforcement rules |
| `entity.md` | Doc Entity index for this crate's documentation scope |
| `doc_graph.yml` | Cross-reference graph for all doc instances |

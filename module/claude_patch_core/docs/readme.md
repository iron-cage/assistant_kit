# docs/

### Scope

**Responsibilities:** Documentation for the `claude_patch_core` crate covering the patch-component domain model, install/uninstall/pin/unpin state semantics, and non-functional constraints.
**In Scope:** Feature requirements (`feature/`), pin-blocks-uninstall constraint (`invariant/`), and doc cross-reference graph.
**Out of Scope:** Source code (→ `src/`), automated tests (→ `tests/`), CLI command design (→ `claude_patch/docs/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `feature/` | Patch component model design and install/uninstall/pin/unpin semantics |
| `invariant/` | Pin-blocks-uninstall constraint and enforcement rules |
| `entity.md` | Doc Entity index for this crate's documentation scope |
| `doc_graph.yml` | Cross-reference graph for all doc instances |

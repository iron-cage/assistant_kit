# Invariant Doc Entity

### Scope

- **Purpose**: Document structural constraints that `claude_version_core` must always satisfy.
- **Responsibility**: Index of invariant doc instances covering the Layer 1 boundary and cross-file literal consistency.
- **In Scope**: Dependency restrictions, error-type boundary, documentation lints, pinned-literal synchronization.
- **Out of Scope**: API contracts (→ `api/`), CLI behavior and algorithms (→ `../../../claude_version/docs/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Layer 1 Boundary](001_layer_one_boundary.md) | Depends only on `claude_core`; `CoreError` never `ErrorData` | ✅ |
| 002 | [Alias Literal Consistency](002_alias_literal_consistency.md) | Pinned `stable` value stays synchronized across its mirror sites | ✅ |
| — | [procedure.md](procedure.md) | Workflow for creating and updating invariant doc instances | ✅ |

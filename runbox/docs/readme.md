# container runner

Variability analysis of the container test infrastructure.

### Scope

- **Purpose:** Map the variability dimensions of the container test infrastructure.
- **Responsibility:** Per-instance reference collections for every configurable parameter slot and swappable plugin slot.
- **In Scope:** Scalar parameters (`runbox.yml` keys, required or optional-with-default); plugin slots (hook-based ✅ and param-based 🔧).
- **Out of Scope:** Container runner implementation code (→ `runbox/`); Docker build stages (→ `runbox/runbox.dockerfile`); test results.

### Responsibility Table

| File/Dir | Responsibility |
|----------|----------------|
| `procedure.md` | Add a new analysis dimension (new sub-collection type) |
| `parameter/` | Per-parameter reference for all scalar configuration slots |
| `plugin/` | Per-plugin reference for all swappable behavioral slots |

# runbox

Variability analysis of the runbox test container infrastructure.

### Scope

- **Purpose:** Map the variability dimensions of the runbox Docker test infrastructure.
- **Responsibility:** Per-instance reference collections for every configurable parameter slot and swappable plugin slot.
- **In Scope:** Scalar parameters (`runbox.yml` keys and hardcoded scalars); plugin slots (configurable and hardcoded-but-swappable).
- **Out of Scope:** Runbox implementation code (→ `run/`); Docker build stages (→ `run/runbox.dockerfile`); test results.

### Responsibility Table

| File/Dir | Responsibility |
|----------|----------------|
| `procedure.md` | Add a new analysis dimension (new sub-collection type) |
| `parameter/` | Per-parameter reference for all scalar configuration slots |
| `plugin/` | Per-plugin reference for all swappable behavioral slots |

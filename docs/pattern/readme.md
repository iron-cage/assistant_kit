# Pattern Doc Entity

### Scope

- **Purpose**: Document architectural patterns governing the dream workspace structure.
- **Responsibility**: Index of pattern doc instances covering the four-layer crate dependency hierarchy.
- **In Scope**: Layer definitions, permitted dependency directions, Layer Invariant, crate-to-layer assignments.
- **Out of Scope**: Feature behavior (→ `feature/`), invariants (→ `invariant/`), cross-workspace protocol (→ `integration/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Crate Layering](001_crate_layering.md) | Four-layer crate dependency hierarchy and Layer Invariant | ✅ |

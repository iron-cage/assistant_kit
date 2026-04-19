# Invariant Doc Entity

### Scope

- **Purpose**: Document verification approach for structural constraints that `dream` must always satisfy.
- **Responsibility**: Index of invariant test lenses covering the zero-own-logic verification.
- **In Scope**: Grep-based INV-1 enforcement check, compile-time INV-2 verification, `Cargo.toml` INV-3 inspection.
- **Out of Scope**: Feature-gate behavior (→ `feature/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Zero Own Logic](001_no_own_logic.md) | Verification approach for INV-1, INV-2, INV-3 | ✅ |

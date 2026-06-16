# Invariant Tests

Test cases verifying measurable architectural constraints from `docs/invariant/`. Each spec covers
one invariant doc instance and asserts the invariant holds at normal and boundary conditions.

### Scope

- **Purpose**: Verify that each architectural invariant is detectable and enforceable via tooling or test.
- **Responsibility**: Index of per-invariant property assertion files (IN-N entries).
- **In Scope**: All 7 invariants from `docs/invariant/`.
- **Out of Scope**: Feature behavior tests (→ `../feature/`), CLI edge cases (→ `../cli/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `01_zero_third_party_deps.md` | IN- spec for zero crates.io dependency invariant |
| `02_cross_platform.md` | IN- spec for cross-platform path operation invariant |
| `03_clear_errors.md` | IN- spec for actionable error message invariant |
| `04_no_process_execution.md` | IN- spec for zero process execution invariant |
| `05_atomic_switching.md` | IN- spec for atomic account switching invariant |
| `06_param_defaults.md` | IN- spec for parameter default context invariant |
| `07_json_storage_format.md` | IN- spec for JSON pretty-print + trailing newline invariant |

### Coverage Summary

| Invariant | Source | Cases | Status |
|-----------|--------|-------|--------|
| 01_zero_third_party_deps | [docs/invariant/001_zero_third_party_deps.md](../../../docs/invariant/001_zero_third_party_deps.md) | IN-1 … IN-2 | ✅ |
| 02_cross_platform | [docs/invariant/002_cross_platform.md](../../../docs/invariant/002_cross_platform.md) | IN-1 … IN-2 | ✅ |
| 03_clear_errors | [docs/invariant/003_clear_errors.md](../../../docs/invariant/003_clear_errors.md) | IN-1 … IN-2 | ✅ |
| 04_no_process_execution | [docs/invariant/004_no_process_execution.md](../../../docs/invariant/004_no_process_execution.md) | IN-1 … IN-2 | ✅ |
| 05_atomic_switching | [docs/invariant/005_atomic_switching.md](../../../docs/invariant/005_atomic_switching.md) | IN-1 … IN-2 | ✅ |
| 06_param_defaults | [docs/invariant/006_param_defaults.md](../../../docs/invariant/006_param_defaults.md) | IN-1 … IN-2 | ✅ |
| 07_json_storage_format | [docs/invariant/007_json_storage_format.md](../../../docs/invariant/007_json_storage_format.md) | IN-1 … IN-2 | ✅ |

**Total:** 7 specs, 14 IN cases minimum.

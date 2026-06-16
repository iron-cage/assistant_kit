# Test: `unset::`

Edge case coverage for the `unset::` parameter. See [param/12_unset.md](../../../../docs/cli/param/12_unset.md) for specification.

### Scope

- **Purpose**: Edge case tests for the `unset::` parameter.
- **Responsibility**: Boundary values, invalid inputs, mutual exclusion, and default behavior for `unset::`.
- **Commands:** `.config`
- **In Scope**: Single-parameter edge cases, validation errors, mutual exclusion with `value::`.
- **Out of Scope**: Command integration (-> `../command/`), group interactions (-> `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `key::K unset::1` removes key from settings | Valid: unset |
| EC-2 | `key::K unset::1` for nonexistent key -> exit 0 (idempotent) | Valid: idempotent |
| EC-3 | `unset::1` without `key::` -> exit 1, key required | Missing Required |
| EC-4 | `key::K value::V unset::1` -> exit 1, mutually exclusive | Mutual Exclusion |
| EC-5 | `unset::0` (explicit disable) -> no effect, treated as normal mode | Valid: disabled |
| EC-6 | `unset::2` -> exit 1, boolean must be 0 or 1 | Invalid Value |
| EC-7 | `key::K unset::1 dry::1` -> preview without deleting | Dry Run |

## Test Coverage Summary

- Valid (unset): 1 test
- Valid (idempotent): 1 test
- Valid (disabled): 1 test
- Missing Required: 1 test
- Mutual Exclusion: 1 test
- Invalid Value: 1 test
- Dry Run: 1 test

**Total:** 7 edge cases

**Behavioral Divergence Pair:** EC-1 (`key::K unset::1` → removes key, exit 0) ↔ EC-5 (`key::K unset::0` → no effect on settings, exit 0)

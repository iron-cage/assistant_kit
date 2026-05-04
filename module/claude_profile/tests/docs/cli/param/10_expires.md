# Parameter :: `expires::`

Edge case tests for the `expires::` parameter. Tests validate boolean enforcement, default behavior, and Token expiry duration display control.

**Source:** [params.md#parameter--10-expires](../../../../docs/cli/params.md#parameter--10-expires)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `expires:::0` suppressed from output | Field Control |
| EC-2 | `expires::::2` rejected (out of range) | Boundary Values |
| EC-3 | `expires::::yes` rejected (type validation) | Type Validation |
| EC-4 | Default value is `1` (present by default) | Default |
| EC-5 | `expires::::1` explicit enable accepted | Field Control |
| EC-6 | `format::json` output unaffected by `expires::::0` | Interaction |

## Test Coverage Summary

- Field Control: 2 tests (EC-1, EC-5)
- Boundary Values: 1 test (EC-2)
- Type Validation: 1 test (EC-3)
- Default: 1 test (EC-4)
- Interaction: 1 test (EC-6)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases
---

### EC-1: `expires:::0` — field suppressed from output:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .accounts expires:::0`
- **Then:** `Token expiry duration` line absent from output
- **Exit:** 0
- **Source:** [params.md#parameter--10-expires](../../../../docs/cli/params.md#parameter--10-expires)
---

### EC-2: `expires::::2` rejected:

- **Given:** clean environment
- **When:** `clp .accounts expires::::2`
- **Then:** `expires:: must be 0 or 1`; exit 1
- **Exit:** 1
- **Source:** [params.md#parameter--10-expires](../../../../docs/cli/params.md#parameter--10-expires)
---

### EC-3: `expires::::yes` rejected:

- **Given:** clean environment
- **When:** `clp .accounts expires::::yes`
- **Then:** `expires:: must be 0 or 1`; exit 1
- **Exit:** 1
- **Source:** [params.md#parameter--10-expires](../../../../docs/cli/params.md#parameter--10-expires)
---

### EC-4: Default value (present by default):

- **Given:** fixture with at least one account in credential store
- **When:** `clp .accounts`
- **Then:** `Token expiry duration` line present in default output
- **Exit:** 0
- **Source:** [params.md#parameter--10-expires](../../../../docs/cli/params.md#parameter--10-expires)
---

### EC-5: `expires::::1` explicit enable accepted:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .accounts expires::::1`
- **Then:** `Token expiry duration` line present in output
- **Exit:** 0
- **Source:** [params.md#parameter--10-expires](../../../../docs/cli/params.md#parameter--10-expires)
---

### EC-6: `format::json` unaffected by `expires::::0`:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .accounts format::json expires::::0`
- **Then:** JSON output contains all fields regardless of `expires::` value
- **Exit:** 0
- **Source:** [params.md#parameter--10-expires](../../../../docs/cli/params.md#parameter--10-expires)

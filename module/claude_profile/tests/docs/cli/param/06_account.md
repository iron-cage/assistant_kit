# Parameter :: `account::`

Edge case tests for the `account::` parameter. Tests validate boolean enforcement, default behavior, and Active account name display control.

**Source:** [params.md#parameter--6-account](../../../../docs/cli/params.md#parameter--6-account)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `account:::0` suppressed from output | Field Control |
| EC-2 | `account::::2` rejected (out of range) | Boundary Values |
| EC-3 | `account::::yes` rejected (type validation) | Type Validation |
| EC-4 | Default value is `1` (present by default) | Default |
| EC-5 | `account::::1` explicit enable accepted | Field Control |
| EC-6 | `format::json` output unaffected by `account::::0` | Interaction |

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

### EC-1: `account:::0` — field suppressed from output:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .credentials.status account:::0`
- **Then:** `Active account name` line absent from output
- **Exit:** 0
- **Source:** [params.md#parameter--6-account](../../../../docs/cli/params.md#parameter--6-account)
---

### EC-2: `account::::2` rejected:

- **Given:** clean environment
- **When:** `clp .credentials.status account::::2`
- **Then:** `account:: must be 0 or 1`; exit 1
- **Exit:** 1
- **Source:** [params.md#parameter--6-account](../../../../docs/cli/params.md#parameter--6-account)
---

### EC-3: `account::::yes` rejected:

- **Given:** clean environment
- **When:** `clp .credentials.status account::::yes`
- **Then:** `account:: must be 0 or 1`; exit 1
- **Exit:** 1
- **Source:** [params.md#parameter--6-account](../../../../docs/cli/params.md#parameter--6-account)
---

### EC-4: Default value (present by default):

- **Given:** fixture with at least one account in credential store
- **When:** `clp .credentials.status`
- **Then:** `Active account name` line present in default output
- **Exit:** 0
- **Source:** [params.md#parameter--6-account](../../../../docs/cli/params.md#parameter--6-account)
---

### EC-5: `account::::1` explicit enable accepted:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .credentials.status account::::1`
- **Then:** `Active account name` line present in output
- **Exit:** 0
- **Source:** [params.md#parameter--6-account](../../../../docs/cli/params.md#parameter--6-account)
---

### EC-6: `format::json` unaffected by `account::::0`:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .credentials.status format::json account::::0`
- **Then:** JSON output contains all fields regardless of `account::` value
- **Exit:** 0
- **Source:** [params.md#parameter--6-account](../../../../docs/cli/params.md#parameter--6-account)

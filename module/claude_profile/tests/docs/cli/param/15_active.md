# Parameter :: `active::`

Edge case tests for the `active::` parameter. Tests validate boolean enforcement, default behavior, and Active/inactive status display control.

**Source:** [params.md#parameter--15-active](../../../../docs/cli/params.md#parameter--15-active)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `active:::0` suppressed from output | Field Control |
| EC-2 | `active::::2` rejected (out of range) | Boundary Values |
| EC-3 | `active::::yes` rejected (type validation) | Type Validation |
| EC-4 | Default value is `1` (present by default) | Default |
| EC-5 | `active::::1` explicit enable accepted | Field Control |
| EC-6 | `format::json` output unaffected by `active::::0` | Interaction |

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

### EC-1: `active:::0` — field suppressed from output:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .accounts active:::0`
- **Then:** `Active/inactive status` line absent from output
- **Exit:** 0
- **Source:** [params.md#parameter--15-active](../../../../docs/cli/params.md#parameter--15-active)
---

### EC-2: `active::::2` rejected:

- **Given:** clean environment
- **When:** `clp .accounts active::::2`
- **Then:** `active:: must be 0 or 1`; exit 1
- **Exit:** 1
- **Source:** [params.md#parameter--15-active](../../../../docs/cli/params.md#parameter--15-active)
---

### EC-3: `active::::yes` rejected:

- **Given:** clean environment
- **When:** `clp .accounts active::::yes`
- **Then:** `active:: must be 0 or 1`; exit 1
- **Exit:** 1
- **Source:** [params.md#parameter--15-active](../../../../docs/cli/params.md#parameter--15-active)
---

### EC-4: Default value (present by default):

- **Given:** fixture with at least one account in credential store
- **When:** `clp .accounts`
- **Then:** `Active/inactive status` line present in default output
- **Exit:** 0
- **Source:** [params.md#parameter--15-active](../../../../docs/cli/params.md#parameter--15-active)
---

### EC-5: `active::::1` explicit enable accepted:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .accounts active::::1`
- **Then:** `Active/inactive status` line present in output
- **Exit:** 0
- **Source:** [params.md#parameter--15-active](../../../../docs/cli/params.md#parameter--15-active)
---

### EC-6: `format::json` unaffected by `active::::0`:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .accounts format::json active::::0`
- **Then:** JSON output contains all fields regardless of `active::` value
- **Exit:** 0
- **Source:** [params.md#parameter--15-active](../../../../docs/cli/params.md#parameter--15-active)

# Parameter :: `org::`

Edge case tests for the `org::` parameter. Tests validate boolean enforcement, default behavior, and Organisation name display control.

**Source:** [params.md#parameter--12-org](../../../../docs/cli/params.md#parameter--12-org)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `org:::0` suppressed from output | Field Control |
| EC-2 | `org::::2` rejected (out of range) | Boundary Values |
| EC-3 | `org::::yes` rejected (type validation) | Type Validation |
| EC-4 | Default value is `1` (present by default) | Default |
| EC-5 | `org::::1` explicit enable accepted | Field Control |
| EC-6 | `format::json` output unaffected by `org::::0` | Interaction |

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

### EC-1: `org:::0` — field suppressed from output:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .accounts org:::0`
- **Then:** `Organisation name` line absent from output
- **Exit:** 0
- **Source:** [params.md#parameter--12-org](../../../../docs/cli/params.md#parameter--12-org)
---

### EC-2: `org::::2` rejected:

- **Given:** clean environment
- **When:** `clp .accounts org::::2`
- **Then:** `org:: must be 0 or 1`; exit 1
- **Exit:** 1
- **Source:** [params.md#parameter--12-org](../../../../docs/cli/params.md#parameter--12-org)
---

### EC-3: `org::::yes` rejected:

- **Given:** clean environment
- **When:** `clp .accounts org::::yes`
- **Then:** `org:: must be 0 or 1`; exit 1
- **Exit:** 1
- **Source:** [params.md#parameter--12-org](../../../../docs/cli/params.md#parameter--12-org)
---

### EC-4: Default value (present by default):

- **Given:** fixture with at least one account in credential store
- **When:** `clp .accounts`
- **Then:** `Organisation name` line present in default output
- **Exit:** 0
- **Source:** [params.md#parameter--12-org](../../../../docs/cli/params.md#parameter--12-org)
---

### EC-5: `org::::1` explicit enable accepted:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .accounts org::::1`
- **Then:** `Organisation name` line present in output
- **Exit:** 0
- **Source:** [params.md#parameter--12-org](../../../../docs/cli/params.md#parameter--12-org)
---

### EC-6: `format::json` unaffected by `org::::0`:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .accounts format::json org::::0`
- **Then:** JSON output contains all fields regardless of `org::` value
- **Exit:** 0
- **Source:** [params.md#parameter--12-org](../../../../docs/cli/params.md#parameter--12-org)

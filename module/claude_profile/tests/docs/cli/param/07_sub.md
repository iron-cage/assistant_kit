# Parameter :: `sub::`

Edge case tests for the `sub::` parameter. Tests validate boolean enforcement, default behavior, and Subscription type display control.

**Source:** [params.md#parameter--7-sub](../../../../docs/cli/params.md#parameter--7-sub)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `sub:::0` suppressed from output | Field Control |
| EC-2 | `sub::::2` rejected (out of range) | Boundary Values |
| EC-3 | `sub::::yes` rejected (type validation) | Type Validation |
| EC-4 | Default value is `1` (present by default) | Default |
| EC-5 | `sub::::1` explicit enable accepted | Field Control |
| EC-6 | `format::json` output unaffected by `sub::::0` | Interaction |

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

### EC-1: `sub:::0` — field suppressed from output:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .accounts sub:::0`
- **Then:** `Subscription type` line absent from output
- **Exit:** 0
- **Source:** [params.md#parameter--7-sub](../../../../docs/cli/params.md#parameter--7-sub)
---

### EC-2: `sub::::2` rejected:

- **Given:** clean environment
- **When:** `clp .accounts sub::::2`
- **Then:** `sub:: must be 0 or 1`; exit 1
- **Exit:** 1
- **Source:** [params.md#parameter--7-sub](../../../../docs/cli/params.md#parameter--7-sub)
---

### EC-3: `sub::::yes` rejected:

- **Given:** clean environment
- **When:** `clp .accounts sub::::yes`
- **Then:** `sub:: must be 0 or 1`; exit 1
- **Exit:** 1
- **Source:** [params.md#parameter--7-sub](../../../../docs/cli/params.md#parameter--7-sub)
---

### EC-4: Default value (present by default):

- **Given:** fixture with at least one account in credential store
- **When:** `clp .accounts`
- **Then:** `Subscription type` line present in default output
- **Exit:** 0
- **Source:** [params.md#parameter--7-sub](../../../../docs/cli/params.md#parameter--7-sub)
---

### EC-5: `sub::::1` explicit enable accepted:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .accounts sub::::1`
- **Then:** `Subscription type` line present in output
- **Exit:** 0
- **Source:** [params.md#parameter--7-sub](../../../../docs/cli/params.md#parameter--7-sub)
---

### EC-6: `format::json` unaffected by `sub::::0`:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .accounts format::json sub::::0`
- **Then:** JSON output contains all fields regardless of `sub::` value
- **Exit:** 0
- **Source:** [params.md#parameter--7-sub](../../../../docs/cli/params.md#parameter--7-sub)

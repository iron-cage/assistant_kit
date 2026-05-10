# Parameter :: `saved::`

Edge case tests for the `saved::` parameter. Tests validate boolean enforcement, default behavior, and Saved account count display control.

**Source:** [params.md#parameter--13-saved](../../../../docs/cli/params.md#parameter--13-saved)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `saved:::1` included in output | Field Control |
| EC-2 | `saved::::2` rejected (out of range) | Boundary Values |
| EC-3 | `saved::::yes` rejected (type validation) | Type Validation |
| EC-4 | Default value is `0` (absent by default (opt-in)) | Default |
| EC-5 | `saved::::1` explicit enable accepted | Field Control |
| EC-6 | `format::json` output unaffected by `saved::::0` | Interaction |

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

### EC-1: `saved:::1` — field included in output:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .credentials.status saved:::1`
- **Then:** `Saved account count` line absent from output
- **Exit:** 0
- **Source:** [params.md#parameter--13-saved](../../../../docs/cli/params.md#parameter--13-saved)
---

### EC-2: `saved::::2` rejected:

- **Given:** clean environment
- **When:** `clp .credentials.status saved::::2`
- **Then:** `saved:: must be 0 or 1`; exit 1
- **Exit:** 1
- **Source:** [params.md#parameter--13-saved](../../../../docs/cli/params.md#parameter--13-saved)
---

### EC-3: `saved::::yes` rejected:

- **Given:** clean environment
- **When:** `clp .credentials.status saved::::yes`
- **Then:** `saved:: must be 0 or 1`; exit 1
- **Exit:** 1
- **Source:** [params.md#parameter--13-saved](../../../../docs/cli/params.md#parameter--13-saved)
---

### EC-4: Default value (absent by default (opt-in)):

- **Given:** fixture with at least one account in credential store
- **When:** `clp .credentials.status`
- **Then:** `Saved account count` line absent in default output
- **Exit:** 0
- **Source:** [params.md#parameter--13-saved](../../../../docs/cli/params.md#parameter--13-saved)
---

### EC-5: `saved::::1` explicit enable accepted:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .credentials.status saved::::1`
- **Then:** `Saved account count` line present in output
- **Exit:** 0
- **Source:** [params.md#parameter--13-saved](../../../../docs/cli/params.md#parameter--13-saved)
---

### EC-6: `format::json` unaffected by `saved::::0`:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .credentials.status format::json saved::::0`
- **Then:** JSON output contains all fields regardless of `saved::` value
- **Exit:** 0
- **Source:** [params.md#parameter--13-saved](../../../../docs/cli/params.md#parameter--13-saved)

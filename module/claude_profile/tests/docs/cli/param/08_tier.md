# Parameter :: `tier::`

Edge case tests for the `tier::` parameter. Tests validate boolean enforcement, default behavior, and Rate-limit tier display control.

**Source:** [params.md#parameter--8-tier](../../../../docs/cli/params.md#parameter--8-tier)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `tier:::0` suppressed from output | Field Control |
| EC-2 | `tier::::2` rejected (out of range) | Boundary Values |
| EC-3 | `tier::::yes` rejected (type validation) | Type Validation |
| EC-4 | Default value is `1` (present by default) | Default |
| EC-5 | `tier::::1` explicit enable accepted | Field Control |
| EC-6 | `format::json` output unaffected by `tier::::0` | Interaction |

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

### EC-1: `tier:::0` — field suppressed from output:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .accounts tier:::0`
- **Then:** `Rate-limit tier` line absent from output
- **Exit:** 0
- **Source:** [params.md#parameter--8-tier](../../../../docs/cli/params.md#parameter--8-tier)
---

### EC-2: `tier::::2` rejected:

- **Given:** clean environment
- **When:** `clp .accounts tier::::2`
- **Then:** `tier:: must be 0 or 1`; exit 1
- **Exit:** 1
- **Source:** [params.md#parameter--8-tier](../../../../docs/cli/params.md#parameter--8-tier)
---

### EC-3: `tier::::yes` rejected:

- **Given:** clean environment
- **When:** `clp .accounts tier::::yes`
- **Then:** `tier:: must be 0 or 1`; exit 1
- **Exit:** 1
- **Source:** [params.md#parameter--8-tier](../../../../docs/cli/params.md#parameter--8-tier)
---

### EC-4: Default value (present by default):

- **Given:** fixture with at least one account in credential store
- **When:** `clp .accounts`
- **Then:** `Rate-limit tier` line present in default output
- **Exit:** 0
- **Source:** [params.md#parameter--8-tier](../../../../docs/cli/params.md#parameter--8-tier)
---

### EC-5: `tier::::1` explicit enable accepted:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .accounts tier::::1`
- **Then:** `Rate-limit tier` line present in output
- **Exit:** 0
- **Source:** [params.md#parameter--8-tier](../../../../docs/cli/params.md#parameter--8-tier)
---

### EC-6: `format::json` unaffected by `tier::::0`:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .accounts format::json tier::::0`
- **Then:** JSON output contains all fields regardless of `tier::` value
- **Exit:** 0
- **Source:** [params.md#parameter--8-tier](../../../../docs/cli/params.md#parameter--8-tier)

# Parameter :: `file::`

Edge case tests for the `file::` parameter. Tests validate boolean enforcement, default behavior, and Credentials file path display control.

**Source:** [params.md#parameter--12-file](../../../../docs/cli/params.md#parameter--12-file)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `file:::1` included in output | Field Control |
| EC-2 | `file::::2` rejected (out of range) | Boundary Values |
| EC-3 | `file::::yes` rejected (type validation) | Type Validation |
| EC-4 | Default value is `0` (absent by default (opt-in)) | Default |
| EC-5 | `file::::1` explicit enable accepted | Field Control |
| EC-6 | `format::json` output unaffected by `file::::0` | Interaction |

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

### EC-1: `file:::1` — field included in output:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .credentials.status file:::1`
- **Then:** `Credentials file path` line absent from output
- **Exit:** 0
- **Source:** [params.md#parameter--12-file](../../../../docs/cli/params.md#parameter--12-file)
---

### EC-2: `file::::2` rejected:

- **Given:** clean environment
- **When:** `clp .credentials.status file::::2`
- **Then:** `file:: must be 0 or 1`; exit 1
- **Exit:** 1
- **Source:** [params.md#parameter--12-file](../../../../docs/cli/params.md#parameter--12-file)
---

### EC-3: `file::::yes` rejected:

- **Given:** clean environment
- **When:** `clp .credentials.status file::::yes`
- **Then:** `file:: must be 0 or 1`; exit 1
- **Exit:** 1
- **Source:** [params.md#parameter--12-file](../../../../docs/cli/params.md#parameter--12-file)
---

### EC-4: Default value (absent by default (opt-in)):

- **Given:** fixture with at least one account in credential store
- **When:** `clp .credentials.status`
- **Then:** `Credentials file path` line absent in default output
- **Exit:** 0
- **Source:** [params.md#parameter--12-file](../../../../docs/cli/params.md#parameter--12-file)
---

### EC-5: `file::::1` explicit enable accepted:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .credentials.status file::::1`
- **Then:** `Credentials file path` line present in output
- **Exit:** 0
- **Source:** [params.md#parameter--12-file](../../../../docs/cli/params.md#parameter--12-file)
---

### EC-6: `format::json` unaffected by `file::::0`:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .credentials.status format::json file::::0`
- **Then:** JSON output contains all fields regardless of `file::` value
- **Exit:** 0
- **Source:** [params.md#parameter--12-file](../../../../docs/cli/params.md#parameter--12-file)

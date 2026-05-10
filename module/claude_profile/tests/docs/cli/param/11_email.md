# Parameter :: `email::`

Edge case tests for the `email::` parameter. Tests validate boolean enforcement, default behavior, and email address display control on both `.credentials.status` and `.accounts`.

**Source:** [params.md#parameter--11-email](../../../../docs/cli/params.md#parameter--11-email)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `email::0` suppressed from `.credentials.status` output | Field Control |
| EC-2 | `email::0` suppressed from `.accounts` output | Field Control |
| EC-3 | `email::::2` rejected (out of range) | Boundary Values |
| EC-4 | `email::::yes` rejected (type validation) | Type Validation |
| EC-5 | Default value `1` — present in `.credentials.status` by default | Default |
| EC-6 | Default value `1` — present in `.accounts` by default | Default |
| EC-7 | `email::::1` explicit enable accepted | Field Control |
| EC-8 | `format::json` output unaffected by `email::::0` | Interaction |

## Test Coverage Summary

- Field Control: 3 tests (EC-1, EC-2, EC-7)
- Boundary Values: 1 test (EC-3)
- Type Validation: 1 test (EC-4)
- Default: 2 tests (EC-5, EC-6)
- Interaction: 1 test (EC-8)

**Total:** 8 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-3 (invalid/rejected path)

## Test Cases
---

### EC-1: `email::0` — field suppressed from `.credentials.status`:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .credentials.status email::0`
- **Then:** `Email:` line absent from output
- **Exit:** 0
- **Source:** [params.md#parameter--11-email](../../../../docs/cli/params.md#parameter--11-email)
---

### EC-2: `email::0` — field suppressed from `.accounts`:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .accounts email::0`
- **Then:** `Email:` line absent from each account block
- **Exit:** 0
- **Source:** [params.md#parameter--11-email](../../../../docs/cli/params.md#parameter--11-email)
---

### EC-3: `email::::2` rejected:

- **Given:** clean environment
- **When:** `clp .credentials.status email::::2`
- **Then:** `email:: must be 0 or 1`; exit 1
- **Exit:** 1
- **Source:** [params.md#parameter--11-email](../../../../docs/cli/params.md#parameter--11-email)
---

### EC-4: `email::::yes` rejected:

- **Given:** clean environment
- **When:** `clp .credentials.status email::::yes`
- **Then:** `email:: must be 0 or 1`; exit 1
- **Exit:** 1
- **Source:** [params.md#parameter--11-email](../../../../docs/cli/params.md#parameter--11-email)
---

### EC-5: Default value — present in `.credentials.status` by default:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .credentials.status`
- **Then:** `Email:` line present in default output
- **Exit:** 0
- **Source:** [params.md#parameter--11-email](../../../../docs/cli/params.md#parameter--11-email)
---

### EC-6: Default value — present in `.accounts` by default:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .accounts`
- **Then:** `Email:` line present in each account block by default
- **Exit:** 0
- **Source:** [params.md#parameter--11-email](../../../../docs/cli/params.md#parameter--11-email)
---

### EC-7: `email::::1` explicit enable accepted:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .credentials.status email::::1`
- **Then:** `Email:` line present in output
- **Exit:** 0
- **Source:** [params.md#parameter--11-email](../../../../docs/cli/params.md#parameter--11-email)
---

### EC-8: `format::json` unaffected by `email::::0`:

- **Given:** fixture with at least one account in credential store
- **When:** `clp .credentials.status format::json email::::0`
- **Then:** JSON output contains `email` key regardless of `email::` value
- **Exit:** 0
- **Source:** [params.md#parameter--11-email](../../../../docs/cli/params.md#parameter--11-email)

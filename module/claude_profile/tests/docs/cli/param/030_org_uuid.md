# Parameter :: `org_uuid::`

Edge case tests for the `org_uuid::` parameter. Tests validate boolean enforcement, default behavior, and organisation UUID field control from `{name}.json`. Used by `.accounts` (saved credential store snapshot) and `.credentials.status` (active account's `{active_account}.json`).

**Source:** [params.md#parameter--30-org_uuid](../../../../docs/cli/param/030_org_uuid.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `org_uuid::1` shows `Org ID:` line with organization_uuid value | Field Control |
| EC-2 | `org_uuid::2` rejected (out of range) | Boundary Values |
| EC-3 | `org_uuid::yes` rejected (type validation) | Type Validation |
| EC-4 | Default value is `0` (absent by default, opt-in) | Default |
| EC-5 | `org_uuid::0` explicit disable accepted — field absent | Field Control |
| EC-6 | `format::json` always emits `organization_uuid` key regardless of `org_uuid::0` | Interaction |
| EC-7 | Missing `{name}.json` snapshot → `Org ID: N/A` | Edge Case |

## Test Coverage Summary

- Field Control: 2 tests (EC-1, EC-5)
- Boundary Values: 1 test (EC-2)
- Type Validation: 1 test (EC-3)
- Default: 1 test (EC-4)
- Interaction: 1 test (EC-6)
- Edge Case: 1 test (EC-7)

**Total:** 7 edge cases

**Behavioral Divergence Pair:** EC-1 (opt-in enabled) ↔ EC-4 (absent by default)

## Test Cases
---

### EC-1: `org_uuid::1` — Org ID: field included in output

- **Given:** Active account with `{credential_store}/{active_account}.json` containing `{"organization_uuid":"org-xyz-789","organization_name":"Acme Corp"}`
- **When:** `clp .credentials.status org_uuid::1`
- **Then:** Output contains `Org ID:` line with value `org-xyz-789`
- **Exit:** 0
- **Source:** [params.md#parameter--30-org_uuid](../../../../docs/cli/param/030_org_uuid.md)
---

### EC-2: `org_uuid::2` rejected

- **Given:** clean environment with valid credentials
- **When:** `clp .credentials.status org_uuid::2`
- **Then:** Exit 1 with error referencing `org_uuid::`; must be 0 or 1
- **Exit:** 1
- **Source:** [params.md#parameter--30-org_uuid](../../../../docs/cli/param/030_org_uuid.md)
---

### EC-3: `org_uuid::yes` rejected

- **Given:** clean environment with valid credentials
- **When:** `clp .credentials.status org_uuid::yes`
- **Then:** Exit 1 with type validation error referencing `org_uuid::`
- **Exit:** 1
- **Source:** [params.md#parameter--30-org_uuid](../../../../docs/cli/param/030_org_uuid.md)
---

### EC-4: Default value (absent by default, opt-in)

- **Given:** Active account with `{name}.json` containing organization_uuid
- **When:** `clp .credentials.status` (no `org_uuid::` param)
- **Then:** `Org ID:` line absent from output; organization_uuid not exposed unless opted in
- **Exit:** 0
- **Source:** [params.md#parameter--30-org_uuid](../../../../docs/cli/param/030_org_uuid.md)
---

### EC-5: `org_uuid::0` explicit disable accepted — field absent

- **Given:** Active account with `{name}.json` containing organization_uuid
- **When:** `clp .credentials.status org_uuid::0`
- **Then:** `Org ID:` line absent from output; explicit 0 same as default-off
- **Exit:** 0
- **Source:** [params.md#parameter--30-org_uuid](../../../../docs/cli/param/030_org_uuid.md)
---

### EC-6: `format::json` always emits `organization_uuid` key

- **Given:** Active account with `{name}.json` containing `organization_uuid="org-xyz"`
- **When:** `clp .credentials.status format::json org_uuid::0`
- **Then:** JSON output contains `"organization_uuid"` key with value `"org-xyz"` regardless of `org_uuid::0`
- **Exit:** 0
- **Source:** [params.md#parameter--30-org_uuid](../../../../docs/cli/param/030_org_uuid.md)
---

### EC-7: Missing `{name}.json` → `Org ID: N/A`

- **Given:** Active account set via per-machine active marker but no `{active_account}.json` file in credential store
- **When:** `clp .credentials.status org_uuid::1`
- **Then:** Output contains `Org ID:  N/A`
- **Exit:** 0
- **Source:** [params.md#parameter--30-org_uuid](../../../../docs/cli/param/030_org_uuid.md)

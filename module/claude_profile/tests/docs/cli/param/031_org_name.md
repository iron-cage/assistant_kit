# Parameter :: `org_name::`

Edge case tests for the `org_name::` parameter. Tests validate boolean enforcement, default behavior, and organisation display name field control from `{name}.roles.json`. Used by `.accounts` (saved credential store snapshot) and `.credentials.status` (active account's `{_active}.roles.json`).

**Source:** [params.md#parameter--31-org_name](../../../../docs/cli/param/031_org_name.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `org_name::1` shows `Org:` line with organization_name value | Field Control |
| EC-2 | `org_name::2` rejected (out of range) | Boundary Values |
| EC-3 | `org_name::yes` rejected (type validation) | Type Validation |
| EC-4 | Default value is `0` (absent by default, opt-in) | Default |
| EC-5 | `org_name::0` explicit disable accepted — field absent | Field Control |
| EC-6 | `format::json` always emits `organization_name` key regardless of `org_name::0` | Interaction |
| EC-7 | Missing `roles.json` snapshot → `Org: N/A` | Edge Case |

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

### EC-1: `org_name::1` — Org: field included in output

- **Given:** Active account with `{credential_store}/{_active}.roles.json` containing `{"organization_uuid":"org-xyz-789","organization_name":"Acme Corp"}`
- **When:** `clp .credentials.status org_name::1`
- **Then:** Output contains `Org:` line with value `Acme Corp`
- **Exit:** 0
- **Source:** [params.md#parameter--31-org_name](../../../../docs/cli/param/031_org_name.md)
---

### EC-2: `org_name::2` rejected

- **Given:** clean environment with valid credentials
- **When:** `clp .credentials.status org_name::2`
- **Then:** Exit 1 with error referencing `org_name::`; must be 0 or 1
- **Exit:** 1
- **Source:** [params.md#parameter--31-org_name](../../../../docs/cli/param/031_org_name.md)
---

### EC-3: `org_name::yes` rejected

- **Given:** clean environment with valid credentials
- **When:** `clp .credentials.status org_name::yes`
- **Then:** Exit 1 with type validation error referencing `org_name::`
- **Exit:** 1
- **Source:** [params.md#parameter--31-org_name](../../../../docs/cli/param/031_org_name.md)
---

### EC-4: Default value (absent by default, opt-in)

- **Given:** Active account with `roles.json` containing organization_name
- **When:** `clp .credentials.status` (no `org_name::` param)
- **Then:** `Org:` line absent from output; organization_name not exposed unless opted in
- **Exit:** 0
- **Source:** [params.md#parameter--31-org_name](../../../../docs/cli/param/031_org_name.md)
---

### EC-5: `org_name::0` explicit disable accepted — field absent

- **Given:** Active account with `roles.json` containing organization_name
- **When:** `clp .credentials.status org_name::0`
- **Then:** `Org:` line absent from output; explicit 0 same as default-off
- **Exit:** 0
- **Source:** [params.md#parameter--31-org_name](../../../../docs/cli/param/031_org_name.md)
---

### EC-6: `format::json` always emits `organization_name` key

- **Given:** Active account with `roles.json` containing `organization_name="Acme Corp"`
- **When:** `clp .credentials.status format::json org_name::0`
- **Then:** JSON output contains `"organization_name"` key with value `"Acme Corp"` regardless of `org_name::0`
- **Exit:** 0
- **Source:** [params.md#parameter--31-org_name](../../../../docs/cli/param/031_org_name.md)
---

### EC-7: Missing `roles.json` → `Org: N/A`

- **Given:** Active account set in `_active` marker but no `{_active}.roles.json` file in credential store
- **When:** `clp .credentials.status org_name::1`
- **Then:** Output contains `Org:     N/A`
- **Exit:** 0
- **Source:** [params.md#parameter--31-org_name](../../../../docs/cli/param/031_org_name.md)

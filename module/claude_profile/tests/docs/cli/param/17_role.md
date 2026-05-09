# Parameter :: `role::`

Edge case tests for the `role::` parameter. Tests validate boolean enforcement, default behavior, and organization role field control from `oauthAccount`. Used by `.credentials.status` (live `~/.claude.json`) and `.accounts` (saved `{name}.claude.json` snapshot).

**Source:** [params.md#parameter--17-role](../../../../docs/cli/params.md#parameter--17-role)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `role::1` shows `Role:` line with organizationRole value | Field Control |
| EC-2 | `role::2` rejected (out of range) | Boundary Values |
| EC-3 | `role::yes` rejected (type validation) | Type Validation |
| EC-4 | Default value is `0` (absent by default, opt-in) | Default |
| EC-5 | `role::0` explicit disable accepted — field absent | Field Control |
| EC-6 | `format::json` always emits `role` key regardless of `role::0` | Interaction |

## Test Coverage Summary

- Field Control: 2 tests (EC-1, EC-5)
- Boundary Values: 1 test (EC-2)
- Type Validation: 1 test (EC-3)
- Default: 1 test (EC-4)
- Interaction: 1 test (EC-6)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (opt-in enabled) ↔ EC-4 (absent by default)

## Test Cases
---

### EC-1: `role::1` — Role: field included in output

- **Given:** `~/.claude.json` contains `oauthAccount.organizationRole = "admin"`
- **When:** `clp .credentials.status role::1`
- **Then:** Output contains `Role:` line with value `admin`
- **Exit:** 0
- **Source:** [params.md#parameter--17-role](../../../../docs/cli/params.md#parameter--17-role)
---

### EC-2: `role::2` rejected

- **Given:** clean environment with valid credentials
- **When:** `clp .credentials.status role::2`
- **Then:** Exit 1 with error referencing `role::`; must be 0 or 1
- **Exit:** 1
- **Source:** [params.md#parameter--17-role](../../../../docs/cli/params.md#parameter--17-role)
---

### EC-3: `role::yes` rejected

- **Given:** clean environment with valid credentials
- **When:** `clp .credentials.status role::yes`
- **Then:** Exit 1 with type validation error referencing `role::`
- **Exit:** 1
- **Source:** [params.md#parameter--17-role](../../../../docs/cli/params.md#parameter--17-role)
---

### EC-4: Default value (absent by default, opt-in)

- **Given:** `~/.claude.json` contains `oauthAccount.organizationRole = "admin"`
- **When:** `clp .credentials.status` (no `role::` param)
- **Then:** `Role:` line absent from output; organizationRole not exposed unless opted in
- **Exit:** 0
- **Source:** [params.md#parameter--17-role](../../../../docs/cli/params.md#parameter--17-role)
---

### EC-5: `role::0` explicit disable accepted — field absent

- **Given:** `~/.claude.json` contains `oauthAccount.organizationRole = "admin"`
- **When:** `clp .credentials.status role::0`
- **Then:** `Role:` line absent from output; explicit 0 same as default-off
- **Exit:** 0
- **Source:** [params.md#parameter--17-role](../../../../docs/cli/params.md#parameter--17-role)
---

### EC-6: `format::json` always emits `role` key

- **Given:** `~/.claude.json` contains `oauthAccount.organizationRole = "admin"`
- **When:** `clp .credentials.status format::json role::0`
- **Then:** JSON output contains `"role"` key with value `"admin"` regardless of `role::0`
- **Exit:** 0
- **Source:** [params.md#parameter--17-role](../../../../docs/cli/params.md#parameter--17-role)

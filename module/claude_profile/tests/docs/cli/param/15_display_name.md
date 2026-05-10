# Parameter :: `display_name::`

Edge case tests for the `display_name::` parameter. Tests validate boolean enforcement, default behavior, and display name field control from `oauthAccount`. Used by `.credentials.status` (live `~/.claude.json`) and `.accounts` (saved `{name}.claude.json` snapshot).

**Source:** [params.md#parameter--15-display_name](../../../../docs/cli/params.md#parameter--15-display_name)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `display_name::1` shows `Display:` line with displayName value | Field Control |
| EC-2 | `display_name::2` rejected (out of range) | Boundary Values |
| EC-3 | `display_name::yes` rejected (type validation) | Type Validation |
| EC-4 | Default value is `0` (absent by default, opt-in) | Default |
| EC-5 | `display_name::0` explicit disable accepted — field absent | Field Control |
| EC-6 | `format::json` always emits `display_name` key regardless of `display_name::0` | Interaction |

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

### EC-1: `display_name::1` — Display: field included in output

- **Given:** `~/.claude.json` contains `oauthAccount.displayName = "alice"`
- **When:** `clp .credentials.status display_name::1`
- **Then:** Output contains `Display:` line with value `alice`
- **Exit:** 0
- **Source:** [params.md#parameter--15-display_name](../../../../docs/cli/params.md#parameter--15-display_name)
---

### EC-2: `display_name::2` rejected

- **Given:** clean environment with valid credentials
- **When:** `clp .credentials.status display_name::2`
- **Then:** Exit 1 with error referencing `display_name::`; must be 0 or 1
- **Exit:** 1
- **Source:** [params.md#parameter--15-display_name](../../../../docs/cli/params.md#parameter--15-display_name)
---

### EC-3: `display_name::yes` rejected

- **Given:** clean environment with valid credentials
- **When:** `clp .credentials.status display_name::yes`
- **Then:** Exit 1 with type validation error referencing `display_name::`
- **Exit:** 1
- **Source:** [params.md#parameter--15-display_name](../../../../docs/cli/params.md#parameter--15-display_name)
---

### EC-4: Default value (absent by default, opt-in)

- **Given:** `~/.claude.json` contains `oauthAccount.displayName = "alice"`
- **When:** `clp .credentials.status` (no `display_name::` param)
- **Then:** `Display:` line absent from output; displayName value not exposed unless opted in
- **Exit:** 0
- **Source:** [params.md#parameter--15-display_name](../../../../docs/cli/params.md#parameter--15-display_name)
---

### EC-5: `display_name::0` explicit disable accepted — field absent

- **Given:** `~/.claude.json` contains `oauthAccount.displayName = "alice"`
- **When:** `clp .credentials.status display_name::0`
- **Then:** `Display:` line absent from output; explicit 0 same as default-off
- **Exit:** 0
- **Source:** [params.md#parameter--15-display_name](../../../../docs/cli/params.md#parameter--15-display_name)
---

### EC-6: `format::json` always emits `display_name` key

- **Given:** `~/.claude.json` contains `oauthAccount.displayName = "alice"`
- **When:** `clp .credentials.status format::json display_name::0`
- **Then:** JSON output contains `"display_name"` key with value `"alice"` regardless of `display_name::0`
- **Exit:** 0
- **Source:** [params.md#parameter--15-display_name](../../../../docs/cli/params.md#parameter--15-display_name)

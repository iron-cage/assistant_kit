# Parameter :: `uuid::`

Edge case tests for the `uuid::` parameter. Tests validate boolean enforcement, default behavior, and stable user ID field control from `oauthAccount.taggedId`. Used by `.credentials.status` (live `~/.claude.json`) and `.accounts` (saved `{name}.claude.json` snapshot).

**Source:** [params.md#parameter--28-uuid](../../../../docs/cli/param/028_uuid.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `uuid::1` shows `ID:` line with taggedId value | Field Control |
| EC-2 | `uuid::2` rejected (out of range) | Boundary Values |
| EC-3 | `uuid::yes` rejected (type validation) | Type Validation |
| EC-4 | Default value is `0` (absent by default, opt-in) | Default |
| EC-5 | `uuid::0` explicit disable accepted — field absent | Field Control |
| EC-6 | `format::json` always emits `tagged_id` key regardless of `uuid::0` | Interaction |
| EC-7 | Missing `taggedId` in `.claude.json` → `ID: N/A` | Edge Case |

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

### EC-1: `uuid::1` — ID: field included in output

- **Given:** `~/.claude.json` contains `{"oauthAccount":{"taggedId":"user_abc123"}}`
- **When:** `clp .credentials.status uuid::1`
- **Then:** Output contains `ID:` line with value `user_abc123`
- **Exit:** 0
- **Source:** [params.md#parameter--28-uuid](../../../../docs/cli/param/028_uuid.md)
---

### EC-2: `uuid::2` rejected

- **Given:** clean environment with valid credentials
- **When:** `clp .credentials.status uuid::2`
- **Then:** Exit 1 with error referencing `uuid::`; must be 0 or 1
- **Exit:** 1
- **Source:** [params.md#parameter--28-uuid](../../../../docs/cli/param/028_uuid.md)
---

### EC-3: `uuid::yes` rejected

- **Given:** clean environment with valid credentials
- **When:** `clp .credentials.status uuid::yes`
- **Then:** Exit 1 with type validation error referencing `uuid::`
- **Exit:** 1
- **Source:** [params.md#parameter--28-uuid](../../../../docs/cli/param/028_uuid.md)
---

### EC-4: Default value (absent by default, opt-in)

- **Given:** `~/.claude.json` contains `{"oauthAccount":{"taggedId":"user_abc123"}}`
- **When:** `clp .credentials.status` (no `uuid::` param)
- **Then:** `ID:` line absent from output; taggedId value not exposed unless opted in
- **Exit:** 0
- **Source:** [params.md#parameter--28-uuid](../../../../docs/cli/param/028_uuid.md)
---

### EC-5: `uuid::0` explicit disable accepted — field absent

- **Given:** `~/.claude.json` contains `{"oauthAccount":{"taggedId":"user_abc123"}}`
- **When:** `clp .credentials.status uuid::0`
- **Then:** `ID:` line absent from output; explicit 0 same as default-off
- **Exit:** 0
- **Source:** [params.md#parameter--28-uuid](../../../../docs/cli/param/028_uuid.md)
---

### EC-6: `format::json` always emits `tagged_id` key

- **Given:** `~/.claude.json` contains `{"oauthAccount":{"taggedId":"user_abc123"}}`
- **When:** `clp .credentials.status format::json uuid::0`
- **Then:** JSON output contains `"tagged_id"` key with value `"user_abc123"` regardless of `uuid::0`
- **Exit:** 0
- **Source:** [params.md#parameter--28-uuid](../../../../docs/cli/param/028_uuid.md)
---

### EC-7: Missing `taggedId` in snapshot → `ID: N/A`

- **Given:** `~/.claude.json` present but `oauthAccount.taggedId` absent or null
- **When:** `clp .credentials.status uuid::1`
- **Then:** Output contains `ID:      N/A`
- **Exit:** 0
- **Source:** [params.md#parameter--28-uuid](../../../../docs/cli/param/028_uuid.md)

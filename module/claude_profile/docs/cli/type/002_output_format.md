# Type: 2. `OutputFormat`

**Purpose:** Selects between human-readable text, compact table, and machine-parseable JSON output. Enables pipeline composition via `format::json | jq`; enables at-a-glance multi-account comparison via `format::table`.

**Fundamental Type:** Enum — four named rendering modes plus three `.usage`-only variants

**Constants:**
- `TEXT` — human-readable labeled output (default)
- `JSON` — structured JSON output; all fields serialized regardless of field-presence toggles
- `TABLE` — compact aligned table (`.accounts` only)
- `VALUE` — bare scalar string, no headers or footer (`.usage` only; implied by `get::`)
- `TSV` — tab-separated values with header row (`.usage` only)
- `PLAIN` — text layout with no emoji or ANSI colors (`.usage` only; equivalent to `no_color::1`)
- `DEFAULT = Text`

**Constraints:**
- One of: `text`, `json`, `table` (case-insensitive)
- `table` is accepted only by `.accounts`; other commands reject it with exit 1
- Unknown values rejected with exit 1

**Parsing:**

```
pub fn new( s : &str ) -> Result< Self, String >
```

**Methods:**
- `get() -> &str` — string representation (`"text"`, `"json"`, or `"table"`)
- `is_json() -> bool` — true for JSON format
- `is_text() -> bool` — true for text format
- `is_table() -> bool` — true for table format

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`format::`](../param/002_format.md) | Selects rendering mode |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.accounts`](../command/001_account.md#command--3-accounts) | Account list with text/json/table |
| 2 | [`.token.status`](../command/005_token.md#command--7-tokenstatus) | Token classification output |
| 3 | [`.paths`](../command/004_paths.md#command--8-paths) | Path resolution output |
| 4 | [`.usage`](../command/006_usage.md#command--9-usage) | Multi-account usage output |
| 5 | [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus) | Credential metadata output |
| 6 | [`.account.limits`](../command/001_account.md#command--11-accountlimits) | Quota limits output |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | format::json for structured quota data |
| 2 | [Scripted Pipeline Automation](../user_story/004_scripted_automation.md) | format::json for CI/CD pipeline consumption |
| 3 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | format::json for structured diagnostic comparison |

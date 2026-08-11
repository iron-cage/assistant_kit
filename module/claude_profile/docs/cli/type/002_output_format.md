# Type: 2. `OutputFormat`

**Purpose:** Selects between human-readable text, compact table, and machine-parseable JSON output. Enables pipeline composition via `format::json | jq`; enables at-a-glance multi-account comparison via `format::table`.

**Fundamental Type:** Enum — 3 variants (`src/output.rs:13-21`). The three `.usage`-only modes (`Value`/`Tsv`/`Plain`) belong to a separate, differently-named enum, `UsageOutputFormat` (`src/usage/types.rs:340-352`) — not to `OutputFormat` itself.

**Constants (`OutputFormat`, `src/output.rs:13-21` — used by `.accounts`, `.paths`, `.credentials.status`, `.account.limits`):**
- `TEXT` — human-readable labeled output (default)
- `JSON` — structured JSON output; all fields serialized regardless of field-presence toggles
- `TABLE` — compact aligned table (`.accounts` only)
- `DEFAULT = Text`

**Constants (`UsageOutputFormat`, `src/usage/types.rs:340-352` — `.usage` only, separate enum):**
- `TEXT` — human-readable table (default)
- `JSON` — machine-readable JSON array
- `TSV` — tab-separated values with header row, plain-text status labels
- `PLAIN` — same layout as `TEXT` with no emoji or ANSI sequences (equivalent to `no_color::1`)
- `VALUE` — bare scalar string, no headers or footer; outputs one field for the first row only (implied by `get::`)

**Constraints:**
- One of: `text`, `json`, `table` (case-insensitive)
- `table` is accepted only by `.accounts`; other commands reject it with exit 1
- Unknown values rejected with exit 1

**Parsing:**

```
pub fn from_cmd( cmd : &VerifiedCommand ) -> Result< Self, ErrorData >
```

`OutputOptions::from_cmd()` (`src/output.rs`) — not a method on `OutputFormat` itself; there is no `impl OutputFormat` block and no `OutputFormat::new()`/`get()`. Unrecognized `format::` values return `Err(ErrorData)` with `ErrorCode::ArgumentTypeMismatch`; an absent or non-string value defaults to `OutputFormat::Text`.

**Methods:**
- `OutputOptions::is_table() -> bool` — true when the selected format is `Table`; the only accessor method, and it lives on `OutputOptions`, not `OutputFormat`.
- No `is_json()`/`is_text()`/`get()` methods exist.

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`format::`](../param/002_format.md) | Selects rendering mode |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.accounts`](../command/001_account.md#command-3-accounts) | Account list with text/json/table |
| 2 | [`.paths`](../command/004_paths.md#command-8-paths) | Path resolution output |
| 3 | [`.usage`](../command/006_usage.md#command-9-usage) | Multi-account usage output |
| 4 | [`.credentials.status`](../command/002_credentials.md#command-10-credentialsstatus) | Credential metadata output, including token classification |
| 5 | [`.account.limits`](../command/001_account.md#command-11-accountlimits) | Quota limits output |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | format::json for structured quota data |
| 2 | [Scripted Pipeline Automation](../user_story/004_scripted_automation.md) | format::json for CI/CD pipeline consumption |
| 3 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | format::json for structured diagnostic comparison |

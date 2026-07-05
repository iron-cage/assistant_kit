# Parameter: 2. `format::` / `fmt::`

Selects between human-readable text output and machine-parseable JSON. Text is the default for interactive use; JSON enables pipeline integration.

- **Default:** `text`
- **Alias:** `fmt::` (short form; both accepted at runtime)
- **Constraints:** One of `text`, `json`, `table`, `value`, `tsv`, `plain` (case-insensitive); `table` accepted only on `.accounts`; `value`, `tsv`, `plain` accepted only on `.usage`; `.account.inspect` accepts `text` and `json` only
- **Purpose:** Enables CLI composability — `format::json` output can be piped to `jq` for structured extraction without parsing fragile text layouts.

**Examples:**

```text
format::text   → human-readable labeled output (default)
format::json   → JSON object or array
fmt::json      → same as format::json (short alias)
format::table  → compact one-row-per-account table (.accounts only)
format::value  → bare scalar value with no headers or footer (.usage only; implied by get::)
format::tsv    → tab-separated values with header row (.usage only; status uses text labels)
format::plain  → text layout with no emoji or ANSI colors (.usage only; equivalent to no_color::1)
```

### Referenced Type

| # | Type | Role |
|---|------|------|
| 1 | [OutputFormat](../type/002_output_format.md) | Value type |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Output Control](../param_group/001_output_control.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.accounts`](../command/001_account.md#command-3-accounts) | Account list output format |
| 2 | [`.token.status`](../command/005_token.md#command-7-tokenstatus) | Token status output format |
| 3 | [`.paths`](../command/004_paths.md#command-8-paths) | Path resolution output format |
| 4 | [`.usage`](../command/006_usage.md#command-9-usage) | Usage report output format |
| 5 | [`.credentials.status`](../command/002_credentials.md#command-10-credentialsstatus) | Credential status output format |
| 6 | [`.account.limits`](../command/001_account.md#command-11-accountlimits) | Account limits output format |
| 7 | [`.account.inspect`](../command/001_account.md#command-15-accountinspect) | Account inspection output format |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | `format::json` for structured quota data |
| 2 | [Scripted Pipeline Automation](../user_story/004_scripted_automation.md) | `format::json` for CI/CD pipeline consumption |
| 3 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | `format::json` for structured diagnostic comparison |

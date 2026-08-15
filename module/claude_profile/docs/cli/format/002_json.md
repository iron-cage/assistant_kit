# Format: 2. json

- **ID:** F02
- **Trigger:** `format::json`
- **Scope:** All format-capable commands: `.accounts`, `.paths`, `.usage`, `.credentials.status`, `.account.limits`, `.account.inspect`, `.model`, `.models`, `.provider.select`

### Structure

Single-line JSON. All fields included regardless of field-presence parameters (`sub::`, `tier::`, etc.).

- **Single-record commands** (`.credentials.status`, `.paths`, `.account.limits`, `.account.inspect`, `.model`, `.provider.select`): JSON object `{...}`
- **Multi-record commands** (`.accounts`, `.usage`, `.models`): JSON array `[{...}, {...}]`

Error rows in `.usage` appear as `{"account":"...","error":"..."}` objects in the array.

### Rendering Mechanism

`serde_json` serialization via `data_fmt` JSON renderer — no pretty-printing; all fields serialized regardless of field-presence toggles.

### Example

```bash
clp .credentials.status format::json
# {"subscription":"max","tier":"default_claude_max_20x","token":"valid","expires_in_secs":26640,"email":"alice@acme.com","account":"alice@acme.com","file":"/home/user/.claude/.credentials.json","saved":2}

clp .account.limits format::json
# {"session_pct":62,"session_reset_secs":6480,"weekly_all_pct":41,"weekly_all_reset_secs":302400,"weekly_sonnet_pct":38,"weekly_sonnet_reset_secs":302400}

clp .usage format::json
# [
#   {"account":"alice@example.com","expires_at_ms":1748033040000,"session_5h_left_pct":86,...},
#   {"account":"bob@example.com","expires_at_ms":1748028720000,"session_5h_left_pct":100,...}
# ]
```

**Notes:**
- `format::json` combined with `live::1` exits 1 before any fetch (incompatible combination).
- Field-presence params (`sub::0`, `tier::0`, etc.) are ignored in JSON mode — all fields always appear.

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.accounts`](../command/001_account.md#command-3-accounts) | JSON array of account records |
| 2 | [`.paths`](../command/004_paths.md#command-8-paths) | JSON object with canonical file paths |
| 3 | [`.usage`](../command/006_usage.md#command-9-usage) | JSON array of usage records |
| 4 | [`.credentials.status`](../command/002_credentials.md#command-10-credentialsstatus) | JSON object with credential metadata |
| 5 | [`.account.limits`](../command/001_account.md#command-11-accountlimits) | JSON object with quota percentages |
| 6 | [`.account.inspect`](../command/001_account.md#command-15-accountinspect) | JSON object with identity/subscription/membership fields |
| 7 | [`.model`](../command/007_model.md#command-18-model) | JSON object with scope/path/model/effort_level fields |
| 8 | [`.models`](../command/008_models.md#command-19-models) | JSON array of model records |
| 9 | [`.provider.select`](../command/009_provider.md#command-21-providerselect) | JSON object with selected provider |

### Referenced User Stories

| # | User Story | Relevance |
|---|------------|-----------|
| 1 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | Structured quota data for programmatic analysis |
| 2 | [Scripted Pipeline Automation](../user_story/004_scripted_automation.md) | Machine-parseable output for CI/CD pipelines |
| 3 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Structured diagnostic state for comparison |

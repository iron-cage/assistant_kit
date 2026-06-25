# Group :: 1. Output Control

**Parameters:** `format::`, `get::`
**Pattern:** Read-only output formatting and value extraction
**Purpose:** Controls the presentation layer — serialization format for commands that display information, and single-value extraction for scripting pipelines.

| Parameter | Type | Description |
|-----------|------|-------------|
| [`format::`](../param/002_format.md) | [`OutputFormat`](../type/002_output_format.md) | Output format: `text`, `json`, `table` (`.accounts` only), `value`, `tsv`, `plain` (`.usage` only) |
| [`get::`](../param/045_get.md) | `string` | Extract single column value for first filtered row as bare string; implies `format::value` |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.accounts`](../command/001_account.md#command--3-accounts) | `format::` |
| 2 | [`.token.status`](../command/005_token.md#command--7-tokenstatus) | `format::` |
| 3 | [`.paths`](../command/004_paths.md#command--8-paths) | `format::` |
| 4 | [`.usage`](../command/006_usage.md#command--9-usage) | `format::`, `get::` |
| 5 | [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus) | `format::` |
| 6 | [`.account.limits`](../command/001_account.md#command--11-accountlimits) | `format::` |
| 7 | [`.account.inspect`](../command/001_account.md#command--15-accountinspect) | `format::` |

**Typical Patterns:**

```bash
# Scripting: structured JSON for pipeline consumption
clp .accounts format::json
clp .usage format::json

# Scripting: bare value for shell variable capture
clp .usage only_next::1 get::7d_left      # → "65%"
clp .usage only_active::1 get::account    # → "alice@example.com"

# Interactive: default text for human reading
clp .token.status
clp .usage
```

**Semantic Coherence Test**

> "Does parameter X control **output serialization format or value extraction** for commands that display information?"

| Parameter | In group? | Reason |
|-----------|-----------|--------|
| `format::` | Yes | Controls text/JSON/table/value/tsv/plain serialization |
| `get::` | Yes | Extracts a single scalar value as bare string output |
| `name::` | No | Identifies a target entity, not presentation |
| `threshold::` | No | Controls classification boundary, not presentation |
| `dry::` | No | Controls execution mode, not presentation |
| `assignee::` | No | Mutation param for marker assign/unassign; not a presentation parameter (`active::` REMOVED — Feature 065) |
| `no_color::` | No | Controls symbol rendering, not serialization format (Display Control) |

All members pass. No false inclusions.

**Cross-References**

- [../004_parameter_interactions.md](../004_parameter_interactions.md) — `format::json` override rules
- [../../feature/028_usage_row_filtering.md](../../feature/028_usage_row_filtering.md) — `get::` extraction behavior and empty-result semantics

**Notes**

- `format::json` overrides field-presence params — see [004_parameter_interactions.md](../004_parameter_interactions.md#interaction--2-formatjson-overrides-field-presence-params) for the authoritative rule.
- `format::table` ignores field-presence params and uses fixed columns — see [004_parameter_interactions.md](../004_parameter_interactions.md#interaction--3-formattable-ignores-field-presence-params). Only accepted by `.accounts`.
- `get::` implies `format::value` — the two parameters are mutually reinforcing when set together, not conflicting.
- `format::value`, `format::tsv`, `format::plain` are `.usage`-only format values not accepted by other commands.
- Commands not in this group (`.account.save`, `.account.use`, `.account.delete`, `.account.relogin`) produce fixed single-line confirmation messages not affected by formatting parameters.

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Scripted Pipeline Automation](../user_story/004_scripted_automation.md) | `format::json` and `get::` for CI/CD pipelines |
| 2 | [Multi-Account Quota Monitoring](../user_story/003_quota_monitoring.md) | `format::` for structured quota output |
| 3 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | `format::json` for structured diagnostic comparison |

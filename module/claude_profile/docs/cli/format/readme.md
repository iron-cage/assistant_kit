# Output Formats

clp supports three output format modes, selected via the [`format::`](../param/002_format.md) parameter.

| # | File | Format | Trigger | Scope |
|---|------|--------|---------|-------|
| 1 | [001_text.md](001_text.md) | text | `format::text` (default) | All format-capable commands |
| 2 | [002_json.md](002_json.md) | json | `format::json` | All format-capable commands |
| 3 | [003_table.md](003_table.md) | table | `format::table` | `.accounts` only |

**Total:** 3 formats

**Format-capable commands:** [`.accounts`](../command/001_account.md#command--3-accounts), [`.token.status`](../command/005_token.md#command--7-tokenstatus), [`.paths`](../command/004_paths.md#command--8-paths), [`.usage`](../command/006_usage.md#command--9-usage), [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus), [`.account.limits`](../command/001_account.md#command--11-accountlimits)

Mutation commands (`.account.save`, `.account.use`, `.account.delete`, `.account.relogin`) produce fixed confirmation-text output and do not accept `format::`.

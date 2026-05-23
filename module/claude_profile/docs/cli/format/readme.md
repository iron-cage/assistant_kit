# Output Formats

clp supports three output format modes, selected via the [`format::`](../param/02_format.md) parameter.

| # | File | Format | Trigger | Scope |
|---|------|--------|---------|-------|
| 1 | [01_text.md](01_text.md) | text | `format::text` (default) | All format-capable commands |
| 2 | [02_json.md](02_json.md) | json | `format::json` | All format-capable commands |
| 3 | [03_table.md](03_table.md) | table | `format::table` | `.accounts` only |

**Total:** 3 formats

**Format-capable commands:** [`.accounts`](../command/account.md#command--3-accounts), [`.token.status`](../command/token.md#command--7-tokenstatus), [`.paths`](../command/paths.md#command--8-paths), [`.usage`](../command/usage.md#command--9-usage), [`.credentials.status`](../command/credentials.md#command--10-credentialsstatus), [`.account.limits`](../command/account.md#command--11-accountlimits)

Mutation commands (`.account.save`, `.account.use`, `.account.delete`, `.account.relogin`) produce fixed confirmation-text output and do not accept `format::`.

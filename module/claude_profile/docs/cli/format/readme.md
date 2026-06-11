# Output Formats

clp supports multiple output format modes, selected via the [`format::`](../param/002_format.md) parameter. Three modes have full instance documentation; three are `.usage`-only variants.

| # | File | Format | Trigger | Scope |
|---|------|--------|---------|-------|
| 1 | [001_text.md](001_text.md) | text | `format::text` (default) | All format-capable commands |
| 2 | [002_json.md](002_json.md) | json | `format::json` | All format-capable commands |
| 3 | [003_table.md](003_table.md) | table | `format::table` | `.accounts` only |
| 4 | — | value | `format::value` | `.usage` only; implied by `get::` |
| 5 | — | tsv | `format::tsv` | `.usage` only |
| 6 | — | plain | `format::plain` | `.usage` only; equivalent to `no_color::1` |

**Total:** 3 format instance files (F01–F03); 3 additional `.usage`-only variants (value, tsv, plain)

**Format-capable commands:** [`.accounts`](../command/001_account.md#command--3-accounts), [`.token.status`](../command/005_token.md#command--7-tokenstatus), [`.paths`](../command/004_paths.md#command--8-paths), [`.usage`](../command/006_usage.md#command--9-usage), [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus), [`.account.limits`](../command/001_account.md#command--11-accountlimits)

Mutation commands (`.account.save`, `.account.use`, `.account.delete`, `.account.relogin`) produce fixed confirmation-text output and do not accept `format::`.

### See Also

- [`format::`](../param/002_format.md) — parameter that selects the format
- [`OutputFormat`](../type/002_output_format.md) — type defining all format variants including `.usage`-only modes
- [../command/](../command/readme.md) — commands that accept `format::`
- [../user_story/](../user_story/readme.md) — user stories that specify output format requirements

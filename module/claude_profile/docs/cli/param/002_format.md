# Parameter :: 2. `format::` / `fmt::`

Selects between human-readable text output and machine-parseable JSON. Text is the default for interactive use; JSON enables pipeline integration.

- **Type:** [`OutputFormat`](../type/002_output_format.md)
- **Default:** `text`
- **Alias:** `fmt::` (short form; both accepted at runtime)
- **Constraints:** One of `text`, `json`, `table`, `value`, `tsv`, `plain` (case-insensitive); `table` accepted only on `.accounts`; `value`, `tsv`, `plain` accepted only on `.usage`; `.account.inspect` accepts `text` and `json` only
- **Commands:** [`.accounts`](../command/001_account.md#command--3-accounts), [`.token.status`](../command/005_token.md#command--7-tokenstatus), [`.paths`](../command/004_paths.md#command--8-paths), [`.usage`](../command/006_usage.md#command--9-usage), [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus), [`.account.limits`](../command/001_account.md#command--11-accountlimits), [`.account.inspect`](../command/001_account.md#command--15-accountinspect)
- **Purpose:** Enables CLI composability — `format::json` output can be piped to `jq` for structured extraction without parsing fragile text layouts.
- **Group:** [Output Control](../param_group/001_output_control.md)

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

# Parameter :: 47. `no_color::`

Strips emoji and ANSI color sequences from `.usage` output, producing plain text suitable for log files, non-TTY pipelines, and terminals that do not support color.

- **Type:** `bool`
- **Default:** `0`
- **Constraints:** `0`, `1`, `false`, `true`
- **Commands:** [`.usage`](../command/006_usage.md#command--9-usage)
- **Purpose:** Produce color-free and emoji-free output.
- **Group:** Display Control

**Behavior:** When `no_color::1`, all emoji (🟢, 🟡, 🔴, →, ✓, *) are replaced with plain text equivalents and all ANSI escape sequences are stripped. The table structure (columns, alignment, separators) is preserved. Equivalent to `format::plain` for text output.

**Plain text equivalents:**

| Original | Plain |
|----------|-------|
| `🟢` (status ok) | `ok` |
| `🟡` (status warn) | `warn` |
| `🔴` (status err) | `err` |
| `→` (recommended) | `->` |
| `✓` (current) | `*cur` |
| `*` (active) | `*act` |

**Examples:**

```text
no_color::1      -> plain text output without emoji
no_color::1 get::status -> plain status label: "ok", "warn", or "err"
```

**See Also:** [feature/028_usage_row_filtering.md](../../feature/028_usage_row_filtering.md).

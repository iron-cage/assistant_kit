# Parameter :: 47. `no_color::`

Strips emoji and ANSI color sequences from `.usage` output, producing plain text suitable for log files, non-TTY pipelines, and terminals that do not support color.

- **Default:** `0`
- **Constraints:** `0`, `1`, `false`, `true`
- **Purpose:** Produce color-free and emoji-free output.

**Behavior:** When `no_color::1`, all emoji (🟢, 🟡, 🔴, ✓, *) are replaced with plain text equivalents and all ANSI escape sequences are stripped. The table structure (columns, alignment, separators) is preserved. Equivalent to `format::plain` for text output.

**Plain text equivalents:**

| Original | Plain |
|----------|-------|
| `🟢` (status ok) | `ok` |
| `🟡` (status warn) | `warn` |
| `🔴` (status err) | `err` |
| `✓` (current) | `*cur` |
| `*` (active) | `*act` |

**Examples:**

```text
no_color::1      -> plain text output without emoji
no_color::1 get::status -> plain status label: "ok", "warn", or "err"
```

**See Also:** [feature/028_usage_row_filtering.md](../../feature/028_usage_row_filtering.md).

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Display Control](../param_group/005_display_control.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.usage`](../command/006_usage.md#command--9-usage) | Strip emoji and ANSI from quota table output |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Scripted Pipeline Automation](../user_story/004_scripted_automation.md) | Plain text output for log files and non-TTY pipelines |

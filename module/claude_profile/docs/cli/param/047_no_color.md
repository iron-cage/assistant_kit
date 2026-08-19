# Parameter: 47. `no_color::`

Strips emoji and ANSI color sequences from `.usage` and `.accounts` output, producing plain text suitable for log files, non-TTY pipelines, and terminals that do not support color.

- **Default:** `0`
- **Constraints:** `0`, `1`, `false`, `true`
- **Purpose:** Produce color-free and emoji-free output.

**Behavior:** When `no_color::1`, all emoji and non-ASCII glyphs (🟢, 🟡, 🔴, ⚪, 🔒, ●, →, ✓, *) are replaced with plain text equivalents and all ANSI escape sequences are stripped — the `get::` single-value path included. The table structure (columns, alignment, separators) is preserved. Equivalent to `format::plain` for text output. Machine formats are exempt: `format::tsv` already uses plain labels, and `format::json` never passes through the replacement (it would rewrite user data inside JSON string values).

**Plain text equivalents:**

| Original | Plain |
|----------|-------|
| `🟢` (status ok) | `ok` |
| `🟡` (status warn) | `warn` |
| `🔴` (status err) | `err` |
| `⚪` (status static — redirect backend, Feature 071) | `static` |
| `🔒` (claim-locked account name suffix, Feature 070) | `(locked)` |
| `●` (status column header) | `status` |
| `→` (arrow) | `->` |
| `✓` (current) | `*` |
| `*` (active) | `*` (unchanged — same as the current marker; the two become visually indistinguishable under `no_color::1`) |

**Examples:**

```text
no_color::1      -> plain text output without emoji
no_color::1 get::status -> plain status label: "ok", "warn", "err", or "static"
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
| 1 | [`.usage`](../command/006_usage.md#command-9-usage) | Strip emoji and ANSI from quota table output |
| 2 | [`.accounts`](../command/001_account.md#command-3-accounts) | Strip emoji from identity text/table output (✓/🔒); `format::json` exempt |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Scripted Pipeline Automation](../user_story/004_scripted_automation.md) | Plain text output for log files and non-TTY pipelines |

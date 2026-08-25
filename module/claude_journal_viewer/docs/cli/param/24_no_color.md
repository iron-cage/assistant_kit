# CLI Parameter: no_color

Disable ANSI color codes in table output. When `1`, all output
uses plain text without color escapes. Useful for piping to files
or terminals that don't support ANSI colors. Also triggered by
the `NO_COLOR` environment variable (any non-empty value).

- **Type:** [`Boolean`](../type/08_boolean.md)
- **Default:** 0
- **Required:** No

```bash
clj .list no_color::1                 # Plain text table
clj .tail no_color::1 > /tmp/log.txt  # Pipe-safe output
NO_COLOR=1 clj .stats                 # Via environment
```

The parameter and the environment variable feed the same decision: either one
alone suppresses color, and neither can re-enable it once the other has set it.
Accepted values are `1` and `true`; anything else leaves color on.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`Boolean`](../type/08_boolean.md) | Fundamental | Integer | 0 or 1 |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 5 | [Global](../param_group/05_global.md) | Full |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.list`](../command/01_list.md) | 0 | ANSI colors enabled |
| 2 | [`.tail`](../command/02_tail.md) | 0 | ANSI colors enabled |
| 3 | [`.stats`](../command/03_stats.md) | 0 | ANSI colors enabled |
| 4 | [`.search`](../command/04_search.md) | 0 | ANSI colors enabled |
| 6 | [`.prune`](../command/06_prune.md) | 0 | ANSI colors enabled |
| 7 | [`.status`](../command/07_status.md) | 0 | ANSI colors enabled |
| 8 | [`.export`](../command/08_export.md) | 0 | Applies to `format::table` only |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 5 | [Team Reporting](../user_story/005_team_reporting.md) | Lead |

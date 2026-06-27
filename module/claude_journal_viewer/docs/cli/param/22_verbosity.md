# CLI Parameter: verbosity

Output detail level. Controls how much information is included
in the rendered output. Higher levels include more per-item
detail at the cost of output size.

- **Type:** [`Integer`](../type/04_integer.md)
- **Default:** 1
- **Required:** No

Levels for `.status`:
- `0`: Compact one-line summary (files, size, date range)
- `1`: Standard report (files, size, date range, journal level)
- `2`: Per-file breakdown (individual file sizes and dates)

Levels for `.stats`:
- `0`: Totals only (one summary row)
- `1`: Standard grouped table (default)
- `2`: Extended table with percentile columns (p50/p90/p99 duration)

```bash
clj .status verbosity::0             # One-line summary
clj .status verbosity::2             # Per-file breakdown
clj .stats verbosity::2              # Extended stats with percentiles
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`Integer`](../type/04_integer.md) | Fundamental | Integer | 0, 1, or 2 |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 2 | [Display](../param_group/02_display.md) | Full |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 3 | [`.stats`](../command/03_stats.md) | 1 | Standard grouped table |
| 7 | [`.status`](../command/07_status.md) | 1 | Standard health report |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 4 | [Capacity Planning](../user_story/04_capacity_planning.md) | Developer |

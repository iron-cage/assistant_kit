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

`.status` is the only command that takes `verbosity`.

Values above `2` clamp to `2` rather than erroring: asking for more detail than
exists is a coherent request, and the highest level already answers it in full.
Negative and non-numeric values are typos rather than requests, and exit 1 per
[type/04_integer.md](../type/04_integer.md).

```bash
clj .status verbosity::0             # One-line summary
clj .status verbosity::2             # Per-file breakdown
clj .status verbosity::9             # Same as verbosity::2
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
| 7 | [`.status`](../command/07_status.md) | 1 | Standard health report |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 4 | [Capacity Planning](../user_story/004_capacity_planning.md) | Developer |

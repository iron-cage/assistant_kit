# CLI Parameter: wide

Enable wide table output. When `1`, all available columns are
displayed without truncation, including stdout/stderr preview,
full model name, and full directory path. Terminal width is not
enforced — lines may wrap.

- **Type:** [`Boolean`](../type/08_boolean.md)
- **Default:** 0
- **Required:** No

```bash
clj .list wide::1                     # Full-width table
clj .list wide::1 since::1d          # Wide view of today's events
clj .stats wide::1                    # Extended stats columns
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`Boolean`](../type/08_boolean.md) | Fundamental | Integer | 0 or 1 |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 2 | [Display](../param_group/02_display.md) | Full |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.list`](../command/01_list.md) | 0 | Compact table |
| 3 | [`.stats`](../command/03_stats.md) | 0 | Compact table |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Cost Tracking](../user_story/001_cost_tracking.md) | Developer |

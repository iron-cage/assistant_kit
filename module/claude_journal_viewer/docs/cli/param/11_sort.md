# CLI Parameter: sort

Sort field for event listing. Determines which event field
is used as the sort key. Combined with `reverse` to control
ascending/descending order.

- **Type:** [`SortField`](../type/07_sort_field.md)
- **Default:** time
- **Required:** No

```bash
clj .list sort::time                  # Chronological (default)
clj .list sort::cost reverse::1      # Most expensive first
clj .list sort::duration reverse::1  # Longest running first
clj .list sort::exit                  # Group by exit code
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`SortField`](../type/07_sort_field.md) | Enum | String | One of: time, cost, duration, exit, model, command |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 2 | [Display](../param_group/02_display.md) | Full |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.list`](../command/01_list.md) | time | Chronological order |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Cost Tracking](../user_story/001_cost_tracking.md) | Developer |

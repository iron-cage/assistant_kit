# CLI Parameter: limit

Maximum number of events to display or return. Acts as a cap
after filtering and sorting are applied. Set to `0` for unlimited.

- **Type:** [`Integer`](../type/04_integer.md)
- **Default:** 50
- **Required:** No

```bash
clj .list                             # Default: 50 events
clj .list limit::100                  # Up to 100 events
clj .list limit::0                    # All matching events (no cap)
clj .search pattern::"error" limit::10  # First 10 matches
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`Integer`](../type/04_integer.md) | Fundamental | Integer | Non-negative; 0 = unlimited |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 2 | [Display](../param_group/02_display.md) | Full |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.list`](../command/01_list.md) | 50 | Default cap |
| 4 | [`.search`](../command/04_search.md) | 50 | Default cap |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Cost Tracking](../user_story/001_cost_tracking.md) | Developer |
| 4 | [Capacity Planning](../user_story/004_capacity_planning.md) | Developer |

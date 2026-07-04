# CLI Parameter: since

Time window start — show events newer than this duration ago.
Parsed as a human-friendly duration string (e.g., `1h`, `7d`, `30m`, `4w`, `3M`).

- **Type:** [`Duration`](../type/01_duration.md)
- **Default:** -- (no time lower bound)
- **Required:** No

```bash
clj .list since::1h          # Events from last hour
clj .list since::7d          # Events from last 7 days
clj .stats since::30d        # Stats over 30 days
clj .search pattern::"err" since::1d   # Search last day
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`Duration`](../type/01_duration.md) | Semantic | String | Suffix: s/m/h/d/w/M |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 1 | [Filtering](../param_group/01_filtering.md) | Full |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.list`](../command/01_list.md) | -- | No lower bound |
| 3 | [`.stats`](../command/03_stats.md) | 7d | Stats defaults to last week |
| 4 | [`.search`](../command/04_search.md) | -- | No lower bound |
| 8 | [`.export`](../command/08_export.md) | -- | No lower bound |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Cost Tracking](../user_story/001_cost_tracking.md) | Developer |
| 2 | [Failure Diagnosis](../user_story/002_failure_diagnosis.md) | Developer |
| 4 | [Capacity Planning](../user_story/004_capacity_planning.md) | Developer |

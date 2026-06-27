# CLI Parameter: until

Time window end — show events older than this duration ago.
Combined with `since` to define a time range. When both are absent,
all events match. When only `until` is set, events from the beginning
of time to `until` ago are matched.

- **Type:** [`Duration`](../type/01_duration.md)
- **Default:** -- (no time upper bound)
- **Required:** No

```bash
clj .list since::7d until::1d        # Events from 7 days ago to 1 day ago
clj .stats since::30d until::7d      # Stats for 3 weeks before last week
clj .export until::90d               # Export all events older than 90 days
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
| 1 | [`.list`](../command/01_list.md) | -- | No upper bound |
| 3 | [`.stats`](../command/03_stats.md) | -- | No upper bound |
| 4 | [`.search`](../command/04_search.md) | -- | No upper bound |
| 8 | [`.export`](../command/08_export.md) | -- | No upper bound |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 4 | [Capacity Planning](../user_story/04_capacity_planning.md) | Developer |

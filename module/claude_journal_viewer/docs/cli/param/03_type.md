# CLI Parameter: type

Filter events by event type. Accepts one of the 8 canonical
`EventType` variants. Case-insensitive matching.

- **Type:** [`EventType`](../type/02_event_type.md)
- **Default:** -- (all event types)
- **Required:** No

```bash
clj .list type::execution             # Only execution events
clj .tail type::retry                 # Follow retries in real-time
clj .search pattern::"429" type::retry  # Search retries for 429
clj .stats type::timeout              # Timeout statistics
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`EventType`](../type/02_event_type.md) | Enum | String | One of 8 variants |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 1 | [Filtering](../param_group/01_filtering.md) | Full |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.list`](../command/01_list.md) | -- | All types |
| 2 | [`.tail`](../command/02_tail.md) | -- | All types |
| 3 | [`.stats`](../command/03_stats.md) | execution | Stats defaults to executions |
| 4 | [`.search`](../command/04_search.md) | -- | All types |
| 8 | [`.export`](../command/08_export.md) | -- | All types |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Cost Tracking](../user_story/001_cost_tracking.md) | Developer |
| 2 | [Failure Diagnosis](../user_story/002_failure_diagnosis.md) | Developer |
| 3 | [Automation Audit](../user_story/003_automation_audit.md) | Developer |

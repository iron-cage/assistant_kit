# CLI Parameter: model

Filter events by the Claude model name used. Substring match —
`opus` matches `claude-opus-4-8`, `sonnet` matches `claude-sonnet-5`.
Only events that recorded a `model` field are matched.

- **Type:** [`String`](../type/03_string.md)
- **Default:** -- (all models)
- **Required:** No

```bash
clj .list model::opus                 # All opus invocations
clj .list model::sonnet since::7d    # Sonnet usage last week
clj .stats model::opus by::day       # Daily opus costs
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`String`](../type/03_string.md) | Fundamental | String | Substring match against model field |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 1 | [Filtering](../param_group/01_filtering.md) | Full |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.list`](../command/01_list.md) | -- | All models |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Cost Tracking](../user_story/01_cost_tracking.md) | Developer |
| 4 | [Capacity Planning](../user_story/04_capacity_planning.md) | Developer |

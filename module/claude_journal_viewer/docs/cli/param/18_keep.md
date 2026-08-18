# CLI Parameter: keep

Retention threshold for the `.prune` command. An age duration
(`30d`, `4w`), floored to whole days — files whose `YYYY-MM-DD.jsonl`
filename date is older than the window are deleted. Today's file is
never deleted, so a sub-day duration means "keep only today".

- **Type:** [`RetentionSpec`](../type/11_retention_spec.md)
- **Default:** `30d`
- **Required:** No

```bash
clj .prune keep::30d                 # Delete files older than 30 days
clj .prune                           # Same — 30d is the default
clj .prune keep::4w dry_run::1       # Preview what would be pruned
clj .prune keep::12h                 # Floors to 0 days: keep only today
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`RetentionSpec`](../type/11_retention_spec.md) | Semantic | String | Duration, floored to whole days |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 3 | [Aggregation](../param_group/03_aggregation.md) | Partial (prune only) |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 6 | [`.prune`](../command/06_prune.md) | `30d` | Optional — defaults to 30 days |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 4 | [Capacity Planning](../user_story/004_capacity_planning.md) | Developer |

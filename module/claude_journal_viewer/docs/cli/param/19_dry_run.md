# CLI Parameter: dry_run

When set to `1` (or `true`), `.prune` shows which files would be
deleted without actually deleting them. The output lists each
candidate as a `Would delete: <path>` line, followed by a count.

- **Type:** [`Boolean`](../type/08_boolean.md)
- **Default:** 0
- **Required:** No

```bash
clj .prune keep::30d dry_run::1      # Preview what would be pruned
clj .prune dry_run::1                # Preview with the default 30d window
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`Boolean`](../type/08_boolean.md) | Fundamental | Integer | 0 or 1 |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 3 | [Aggregation](../param_group/03_aggregation.md) | Partial (prune only) |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 6 | [`.prune`](../command/06_prune.md) | 0 | Live deletion by default |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 4 | [Capacity Planning](../user_story/004_capacity_planning.md) | Developer |

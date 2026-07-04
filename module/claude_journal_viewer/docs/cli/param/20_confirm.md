# CLI Parameter: confirm

Skip the interactive confirmation prompt for `.prune`. When `0`
(default), `.prune` displays the list of files to delete and asks
`Delete N files? [y/N]`. When `1`, deletion proceeds without prompting.

- **Type:** [`Boolean`](../type/08_boolean.md)
- **Default:** 0
- **Required:** No

```bash
clj .prune keep::7d confirm::1       # Delete without confirmation
clj .prune keep::30d                  # Interactive confirmation (default)
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
| 6 | [`.prune`](../command/06_prune.md) | 0 | Interactive confirmation |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 4 | [Capacity Planning](../user_story/004_capacity_planning.md) | Developer |

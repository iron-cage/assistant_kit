# CLI Parameter: keep

Retention specification for the `.prune` command. Accepts either
an age duration (`30d`, `4w`, `3M`) or a size limit (`100mb`, `1gb`).
When age-based, files older than the duration are deleted. When
size-based, oldest files are deleted until total size is under the limit.

- **Type:** [`RetentionSpec`](../type/11_retention_spec.md)
- **Default:** -- (none)
- **Required:** Yes (for `.prune`)

```bash
clj .prune keep::30d                  # Delete files older than 30 days
clj .prune keep::100mb               # Delete oldest until under 100MB
clj .prune keep::4w dry_run::1       # Preview what would be pruned
clj .prune keep::1gb confirm::1      # Delete without confirmation
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`RetentionSpec`](../type/11_retention_spec.md) | Semantic | String | Age (duration) or size (bytes with suffix) |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 3 | [Aggregation](../param_group/03_aggregation.md) | Partial (prune only) |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 6 | [`.prune`](../command/06_prune.md) | -- | Required parameter |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 4 | [Capacity Planning](../user_story/04_capacity_planning.md) | Developer |

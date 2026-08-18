# Parameter Group: Aggregation

Stats grouping dimension and prune retention parameters.

### Members

| # | Parameter | Type | Commands |
|---|-----------|------|----------|
| 13 | [`by`](../param/13_by.md) | GroupBy | .stats |
| 18 | [`keep`](../param/18_keep.md) | RetentionSpec | .prune |
| 19 | [`dry_run`](../param/19_dry_run.md) | Boolean | .prune |

### Interaction Rules

- `by` is only used by `.stats` — determines row grouping in the output table
- `keep` and `dry_run` are only used by `.prune`
- `dry_run::1` previews without deleting; there is no confirmation prompt — live deletion is immediate

### Commands

| # | Command | Available Members |
|---|---------|-------------------|
| 3 | [`.stats`](../command/03_stats.md) | by |
| 6 | [`.prune`](../command/06_prune.md) | keep, dry_run |

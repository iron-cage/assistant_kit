# Parameter Group: Aggregation

Stats grouping dimension and prune retention parameters.

### Members

| # | Parameter | Type | Commands |
|---|-----------|------|----------|
| 13 | [`by`](../param/13_by.md) | GroupBy | .stats |
| 18 | [`keep`](../param/18_keep.md) | RetentionSpec | .prune |
| 19 | [`dry_run`](../param/19_dry_run.md) | Boolean | .prune |
| 20 | [`confirm`](../param/20_confirm.md) | Boolean | .prune |

### Interaction Rules

- `by` is only used by `.stats` — determines row grouping in the output table
- `keep`, `dry_run`, `confirm` are only used by `.prune`
- `dry_run::1` overrides `confirm` — dry run never deletes, so confirmation is skipped
- `confirm::1` is only meaningful when `dry_run::0` (the default)

### Commands

| # | Command | Available Members |
|---|---------|-------------------|
| 3 | [`.stats`](../command/03_stats.md) | by |
| 6 | [`.prune`](../command/06_prune.md) | keep, dry_run, confirm |

# Parameter Group: Display

Output format, sort, limit, and column selection parameters.
Control how filtered events are rendered to the user.

### Members

| # | Parameter | Type | Commands |
|---|-----------|------|----------|
| 09 | [`limit`](../param/09_limit.md) | Integer | .list, .search |
| 10 | [`format`](../param/10_format.md) | OutputFormat | .list, .tail, .export |
| 11 | [`sort`](../param/11_sort.md) | SortField | .list |
| 12 | [`reverse`](../param/12_reverse.md) | Boolean | .list |
| 22 | [`verbosity`](../param/22_verbosity.md) | Integer | .stats, .status |
| 23 | [`output`](../param/23_output.md) | Path | .export |
| 25 | [`wide`](../param/25_wide.md) | Boolean | .list, .stats |
| 26 | [`columns`](../param/26_columns.md) | String | .list |

### Interaction Rules

- `sort` and `reverse` are co-dependent: `reverse` only affects the field specified by `sort`
- `wide` and `columns` are mutually exclusive: `wide::1` shows all columns, `columns` selects specific ones; if both set, `columns` takes precedence
- `format::table` is the only format affected by `wide`, `columns`, and `no_color`
- `limit` caps output after sort+reverse are applied
- `output` only applies to `.export` — other commands always write to stdout
- `verbosity` values beyond 0-2 are clamped to 2

### Commands

| # | Command | Available Members |
|---|---------|-------------------|
| 1 | [`.list`](../command/01_list.md) | limit, format, sort, reverse, wide, columns |
| 2 | [`.tail`](../command/02_tail.md) | format |
| 3 | [`.stats`](../command/03_stats.md) | verbosity, wide |
| 7 | [`.status`](../command/07_status.md) | verbosity |
| 8 | [`.export`](../command/08_export.md) | format, output |

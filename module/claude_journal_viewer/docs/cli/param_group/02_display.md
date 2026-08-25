# Parameter Group: Display

Output format, sort, limit, and detail-level parameters.
Control how filtered events are rendered to the user.

### Members

| # | Parameter | Type | Commands |
|---|-----------|------|----------|
| 09 | [`limit`](../param/09_limit.md) | Integer | .list, .stats, .search, .export |
| 10 | [`format`](../param/10_format.md) | OutputFormat | .list, .tail, .export |
| 11 | [`sort`](../param/11_sort.md) | SortField | .list |
| 12 | [`reverse`](../param/12_reverse.md) | Boolean | .list |
| 22 | [`verbosity`](../param/22_verbosity.md) | Integer | .status |
| 23 | [`output`](../param/23_output.md) | Path | .export |
| 29 | [`out`](../param/29_out.md) | Path | .chart |

### Interaction Rules

- `sort` and `reverse` are co-dependent: `reverse` only affects the field specified by `sort`
- `format::table` is the only format affected by `no_color`
- `limit` caps output after sort+reverse are applied on `.list`; on `.stats`, `.search`, and `.export` it caps the events *read*, before aggregation or pattern matching. Only `.list` applies a default (50); the other three are uncapped when it is absent
- `output` and `out` are both destinations and are not interchangeable: `output` is required by `.export` (there is no stdout path), while `out` defaults to `usage.svg` for `.chart` — passing either to the other command exits 1
- `verbosity` values beyond 0-2 are clamped to 2; negative or non-numeric values exit 1

The group carries no column-selection member. `wide` and `columns` (25, 26) were
retracted rather than built: `.list format::csv` and `format::json` already hand
the full field set to `cut` and `jq`, so a table-only column vocabulary would
have been a second, weaker way to ask the same question. `.stats` lost both the
members it had that way (`verbosity` alongside `wide`); it still participates,
but only through `limit`, which reaches it as part of the shared filter
vocabulary rather than as a rendering choice of its own.

### Commands

| # | Command | Available Members |
|---|---------|-------------------|
| 1 | [`.list`](../command/01_list.md) | limit, format, sort, reverse |
| 2 | [`.tail`](../command/02_tail.md) | format |
| 3 | [`.stats`](../command/03_stats.md) | limit |
| 4 | [`.search`](../command/04_search.md) | limit |
| 7 | [`.status`](../command/07_status.md) | verbosity |
| 8 | [`.export`](../command/08_export.md) | limit, format, output |
| 9 | [`.chart`](../command/09_chart.md) | out |

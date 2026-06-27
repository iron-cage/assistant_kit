# CLI Parameters

### Scope

- **Purpose**: Per-parameter detail pages with type, defaults, and command cross-references.
- **Responsibility**: Single source of truth for each parameter's semantics, constraints, and usage context.

All parameters use unilang `param::value` syntax.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `01_since.md` | Time window start filter |
| `02_until.md` | Time window end filter |
| `03_type.md` | Event type filter |
| `04_command.md` | CLR command name filter |
| `05_exit.md` | Exit code filter |
| `06_model.md` | Model name filter |
| `07_dir.md` | Working directory filter |
| `08_creds.md` | Credential name filter |
| `09_limit.md` | Max results cap |
| `10_format.md` | Output format selection |
| `11_sort.md` | Sort field selection |
| `12_reverse.md` | Reverse sort order toggle |
| `13_by.md` | Stats grouping dimension |
| `14_pattern.md` | Regex search pattern |
| `15_port.md` | HTTP server port |
| `16_bind.md` | HTTP server bind address |
| `17_open.md` | Auto-open browser toggle |
| `18_keep.md` | Retention spec for pruning |
| `19_dry_run.md` | Dry run toggle for prune |
| `20_confirm.md` | Skip confirmation prompt |
| `21_journal_dir.md` | Journal directory override |
| `22_verbosity.md` | Output detail level |
| `23_output.md` | Export output file path |
| `24_no_color.md` | Disable ANSI colors |
| `25_wide.md` | Wide table output toggle |
| `26_columns.md` | Column selection for tables |
| `27_refresh.md` | Auto-refresh interval |
| `28_include_stdout.md` | Search in stdout content |

### All Parameters (28 total)

| # | Parameter | Type | Default | Commands |
|---|-----------|------|---------|----------|
| 01 | [`since`](01_since.md) | [Duration](../type/01_duration.md) | -- | .list, .search, .stats, .export |
| 02 | [`until`](02_until.md) | [Duration](../type/01_duration.md) | -- | .list, .search, .stats, .export |
| 03 | [`type`](03_type.md) | [EventType](../type/02_event_type.md) | -- | .list, .tail, .search, .stats, .export |
| 04 | [`command`](04_command.md) | [String](../type/03_string.md) | -- | .list, .search, .stats, .export |
| 05 | [`exit`](05_exit.md) | [Integer](../type/04_integer.md) | -- | .list |
| 06 | [`model`](06_model.md) | [String](../type/03_string.md) | -- | .list |
| 07 | [`dir`](07_dir.md) | [Path](../type/05_path.md) | -- | .list |
| 08 | [`creds`](08_creds.md) | [String](../type/03_string.md) | -- | .list |
| 09 | [`limit`](09_limit.md) | [Integer](../type/04_integer.md) | 50 | .list, .search |
| 10 | [`format`](10_format.md) | [OutputFormat](../type/06_output_format.md) | table | .list, .tail, .export |
| 11 | [`sort`](11_sort.md) | [SortField](../type/07_sort_field.md) | time | .list |
| 12 | [`reverse`](12_reverse.md) | [Boolean](../type/08_boolean.md) | 0 | .list |
| 13 | [`by`](13_by.md) | [GroupBy](../type/09_group_by.md) | day | .stats |
| 14 | [`pattern`](14_pattern.md) | [String](../type/03_string.md) | -- | .search |
| 15 | [`port`](15_port.md) | [Port](../type/10_port.md) | 8411 | .serve |
| 16 | [`bind`](16_bind.md) | [String](../type/03_string.md) | 127.0.0.1 | .serve |
| 17 | [`open`](17_open.md) | [Boolean](../type/08_boolean.md) | 0 | .serve |
| 18 | [`keep`](18_keep.md) | [RetentionSpec](../type/11_retention_spec.md) | -- | .prune |
| 19 | [`dry_run`](19_dry_run.md) | [Boolean](../type/08_boolean.md) | 0 | .prune |
| 20 | [`confirm`](20_confirm.md) | [Boolean](../type/08_boolean.md) | 0 | .prune |
| 21 | [`journal_dir`](21_journal_dir.md) | [Path](../type/05_path.md) | ~/.clr/journal/ | .list, .tail, .search, .stats, .status |
| 22 | [`verbosity`](22_verbosity.md) | [Integer](../type/04_integer.md) | 1 | .stats, .status |
| 23 | [`output`](23_output.md) | [Path](../type/05_path.md) | -- | .export |
| 24 | [`no_color`](24_no_color.md) | [Boolean](../type/08_boolean.md) | 0 | .list, .tail, .stats |
| 25 | [`wide`](25_wide.md) | [Boolean](../type/08_boolean.md) | 0 | .list, .stats |
| 26 | [`columns`](26_columns.md) | [String](../type/03_string.md) | -- | .list |
| 27 | [`refresh`](27_refresh.md) | [Integer](../type/04_integer.md) | 10 | .serve |
| 28 | [`include_stdout`](28_include_stdout.md) | [Boolean](../type/08_boolean.md) | 0 | .search |

**Total:** 28 parameters

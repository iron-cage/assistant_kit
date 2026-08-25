# Parameters

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
| `21_journal_dir.md` | Journal directory override |
| `22_verbosity.md` | Output detail level |
| `23_output.md` | Export output file path |
| `24_no_color.md` | Disable ANSI colors |
| `27_refresh.md` | Auto-refresh interval |
| `28_include_stdout.md` | Tombstone — the flag `.search` superseded |
| `29_out.md` | Chart output file path |

### All Parameters (25 total)

The `Commands` column is the set of commands that **accept** the parameter —
the same set `known_params` builds in `src/cli_main.rs`, and the same set each
parameter's own page lists under Referenced Commands. All three are pinned to
each other by `tests/cli_doc_consistency.rs`, so they cannot drift apart
silently. A command *page* may enumerate fewer than this: every event-reading
command takes the whole filter vocabulary whether or not its page spells all of
it out (see [command/readme.md](../command/readme.md)).

| # | Parameter | Type | Default | Commands |
|---|-----------|------|---------|----------|
| 01 | [`since`](01_since.md) | [Duration](../type/01_duration.md) | -- | .list, .stats, .search, .export |
| 02 | [`until`](02_until.md) | [Duration](../type/01_duration.md) | -- | .list, .tail, .stats, .search, .export |
| 03 | [`type`](03_type.md) | [EventType](../type/02_event_type.md) | -- | .list, .tail, .stats, .search, .export |
| 04 | [`command`](04_command.md) | [String](../type/03_string.md) | -- | .list, .tail, .stats, .search, .export |
| 05 | [`exit`](05_exit.md) | [Integer](../type/04_integer.md) | -- | .list, .tail, .stats, .search, .export |
| 06 | [`model`](06_model.md) | [String](../type/03_string.md) | -- | .list, .tail, .stats, .search, .export |
| 07 | [`dir`](07_dir.md) | [Path](../type/05_path.md) | -- | .list, .tail, .stats, .search, .export |
| 08 | [`creds`](08_creds.md) | [String](../type/03_string.md) | -- | .list, .tail, .stats, .search, .export |
| 09 | [`limit`](09_limit.md) | [Integer](../type/04_integer.md) | 50 on .list; unset elsewhere | .list, .stats, .search, .export |
| 10 | [`format`](10_format.md) | [OutputFormat](../type/06_output_format.md) | table | .list, .tail, .export |
| 11 | [`sort`](11_sort.md) | [SortField](../type/07_sort_field.md) | time | .list |
| 12 | [`reverse`](12_reverse.md) | [Boolean](../type/08_boolean.md) | 0 | .list |
| 13 | [`by`](13_by.md) | [GroupBy](../type/09_group_by.md) | day | .stats |
| 14 | [`pattern`](14_pattern.md) | [String](../type/03_string.md) | -- | .search |
| 15 | [`port`](15_port.md) | [Port](../type/10_port.md) | 0 (OS-assigned) | .serve |
| 16 | [`bind`](16_bind.md) | [String](../type/03_string.md) | 127.0.0.1 | .serve |
| 17 | [`open`](17_open.md) | [Boolean](../type/08_boolean.md) | 0 | .serve, .chart |
| 18 | [`keep`](18_keep.md) | [RetentionSpec](../type/11_retention_spec.md) | -- | .prune |
| 19 | [`dry_run`](19_dry_run.md) | [Boolean](../type/08_boolean.md) | 0 | .prune |
| 21 | [`journal_dir`](21_journal_dir.md) | [Path](../type/05_path.md) | ~/.clr/journal/ | .list, .tail, .stats, .search, .serve, .prune, .status, .export, .chart |
| 22 | [`verbosity`](22_verbosity.md) | [Integer](../type/04_integer.md) | 1 | .status |
| 23 | [`output`](23_output.md) | [Path](../type/05_path.md) | -- | .export |
| 24 | [`no_color`](24_no_color.md) | [Boolean](../type/08_boolean.md) | 0 | .list, .tail, .stats, .search, .serve, .prune, .status, .export, .chart |
| 27 | [`refresh`](27_refresh.md) | [Integer](../type/04_integer.md) | 10 | .serve |
| 29 | [`out`](29_out.md) | [Path](../type/05_path.md) | usage.svg | .chart |

**Total:** 25 parameters — the same 25 [command/readme.md](../command/readme.md)
counts, and now the same *set*: `out` has a page, and `include_stdout` has been
retracted from the live vocabulary rather than left half-documented.

`since` and `limit` are absent from `.tail` on purpose. It follows the journal
forward from now, so there is no earlier event for `since::` to exclude and no
end for `limit::` to stop at — both used to be accepted and applied to nothing.

Numbering keeps historical gaps rather than renumbering, so a cross-reference
written against an earlier revision still resolves to the same parameter:

- `20` was `confirm`, dropped — `.prune` deletes without prompting, and `dry_run` is the preview mechanism
- `25` was `wide` and `26` was `columns`, both retracted — they described a table renderer that was never built, and `format::csv`/`format::json` piped through `cut`/`jq` already covers what they promised
- `28` was `include_stdout`, superseded — `.search` reads `stdout`/`stderr` unconditionally, so there is no narrower default left for the flag to widen. Its page is kept as a tombstone because the surrounding docs link to it; the parameter itself is not accepted and exits 1

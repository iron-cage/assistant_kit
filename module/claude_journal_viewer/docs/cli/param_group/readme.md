# Parameter Groups

### Scope

- **Purpose**: Per-group detail pages with membership and interaction rules.
- **Responsibility**: Define parameter group coherence, co-occurrence rules, and command applicability.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| [01_filtering.md](01_filtering.md) | Time window, event type, and field-match filter params |
| [02_display.md](02_display.md) | Output format, sort, limit, and destination params |
| [03_aggregation.md](03_aggregation.md) | Stats grouping and prune retention params |
| [04_search.md](04_search.md) | Search pattern param |
| [05_global.md](05_global.md) | Cross-command params (journal_dir, no_color, serve config) |

### All Groups (5 total)

| # | Group | Members | Commands |
|---|-------|---------|----------|
| 01 | [Filtering](01_filtering.md) | since, until, type, command, exit, model, dir, creds | .list, .tail, .stats, .search, .export |
| 02 | [Display](02_display.md) | limit, format, sort, reverse, verbosity, output, out | .list, .tail, .stats, .search, .status, .export, .chart |
| 03 | [Aggregation](03_aggregation.md) | by, keep, dry_run | .stats, .prune |
| 04 | [Search](04_search.md) | pattern | .search |
| 05 | [Global](05_global.md) | journal_dir, no_color, port, bind, open, refresh | All commands |

**Total:** 5 groups

Membership is by concept, not by acceptance: `.tail` appears under Filtering but
does not accept `since` or `limit` (it follows forward, so neither has anything
to act on), and Global's `port`/`bind`/`open`/`refresh` reach only `.serve` and
— for `open` — `.chart`. The authoritative per-parameter command set is the
`Commands` column in [param/readme.md](../param/readme.md), which is pinned to
`known_params` by `tests/cli_doc_consistency.rs`.

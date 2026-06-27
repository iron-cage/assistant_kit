# CLI Parameter Groups

### Scope

- **Purpose**: Per-group detail pages with membership and interaction rules.
- **Responsibility**: Define parameter group coherence, co-occurrence rules, and command applicability.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `01_filtering.md` | Time window, event type, and field-match filter params |
| `02_display.md` | Output format, sort, limit, and column params |
| `03_aggregation.md` | Stats grouping and prune retention params |
| `04_search.md` | Regex pattern and search scope params |
| `05_global.md` | Cross-command params (journal_dir, no_color, serve config) |

### All Groups (5 total)

| # | Group | Members | Commands |
|---|-------|---------|----------|
| 01 | [Filtering](01_filtering.md) | since, until, type, command, exit, model, dir, creds | .list, .tail, .search, .stats, .export |
| 02 | [Display](02_display.md) | limit, format, sort, reverse, verbosity, wide, columns, output | .list, .tail, .stats, .export |
| 03 | [Aggregation](03_aggregation.md) | by, keep, dry_run, confirm | .stats, .prune |
| 04 | [Search](04_search.md) | pattern, include_stdout | .search |
| 05 | [Global](05_global.md) | journal_dir, no_color, port, bind, open, refresh | All commands |

**Total:** 5 groups

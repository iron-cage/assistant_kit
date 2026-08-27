# Commands

### All Commands

| # | Command | Params | Description | Example |
|---|---------|-------:|-------------|---------|
| 1 | [`.list`](01_list.md) | 14 | List journal events with filtering and sort | `clj .list since::1h` |
| 2 | [`.tail`](02_tail.md) | 5 | Follow journal events in real-time | `clj .tail type::execution` |
| 3 | [`.stats`](03_stats.md) | 7 | Aggregate statistics (cost, tokens, success rate) | `clj .stats by::model since::7d` |
| 4 | [`.search`](04_search.md) | 7 | Literal substring search across prompt and captured output | `clj .search pattern::"rate limit"` |
| 5 | [`.serve`](05_serve.md) | 6 | Start web viewer on localhost | `clj .serve port::8411` |
| 6 | [`.prune`](06_prune.md) | 4 | Delete journal files older than an age window | `clj .prune keep::30d` |
| 7 | [`.status`](07_status.md) | 3 | Show journal health, size, configuration | `clj .status` |
| 8 | [`.export`](08_export.md) | 8 | Export filtered events to file | `clj .export format::csv since::7d` |
| 9 | [`.chart`](09_chart.md) | 4 | Render a usage SVG chart, optionally opened in browser | `clj .chart out::usage.svg open::1` |

Each `Params` count is the number of rows in that command page's own Parameters
table, including the two global params every command accepts. It is not the size
of the set `known_params` accepts in `src/cli_main.rs`, which is deliberately
wider — every event-reading command accepts the full filter vocabulary whether or
not its page enumerates all of it.

### Quick Reference

- **Total commands:** 9
- **Total unique parameters:** 25 across the command pages above
- **Parameters without defaults:** 10 (since, until, type, command, exit, model, dir, creds, pattern, output)
- **Most-used parameter:** `journal_dir` (9 commands), `no_color` (9 commands), `since` (4 commands)

That 25 is the same 25 [`param/readme.md`](../param/readme.md) counts, and the
same *set*. It was not always: `out` (`.chart`) appeared here with no parameter
page of its own, and `include_stdout` had a page enumerated by no command page,
so the two totals matched by coincidence while the sets differed by one each way.
`out` now has [a page](../param/29_out.md), `include_stdout` is retracted to a
tombstone, and `tests/cli_doc_consistency.rs` fails if either drifts again.

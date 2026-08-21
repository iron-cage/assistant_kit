# Commands

### All Commands

| # | Command | Params | Description | Example |
|---|---------|-------:|-------------|---------|
| 1 | `.list` | 12 | List journal events with filtering and sort | `clj .list since::1h` |
| 2 | `.tail` | 5 | Follow journal events in real-time | `clj .tail type::execution` |
| 3 | `.stats` | 6 | Aggregate statistics (cost, tokens, success rate) | `clj .stats by::model since::7d` |
| 4 | `.search` | 7 | Full-text regex search across event data | `clj .search pattern::"rate limit"` |
| 5 | `.serve` | 4 | Start web viewer on localhost | `clj .serve port::8411` |
| 6 | `.prune` | 2 | Delete journal files older than an age window | `clj .prune keep::30d` |
| 7 | `.status` | 2 | Show journal health, size, configuration | `clj .status` |
| 8 | `.export` | 6 | Export filtered events to file | `clj .export format::csv since::7d` |
| 9 | `.chart` | 3 | Render a usage SVG chart, optionally opened in browser | `clj .chart out::usage.svg open::1` |

### Quick Reference

- **Total commands:** 9
- **Total unique parameters:** 28
- **Parameters without defaults:** 13 (since, until, type, command, exit, model, dir, creds, pattern, keep, columns, output, by)
- **Most-used parameter:** `journal_dir` (8 commands), `since` (4 commands)

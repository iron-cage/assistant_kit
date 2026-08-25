# Parameter Group: Global

Cross-command parameters and serve-specific configuration.

### Members

| # | Parameter | Type | Commands |
|---|-----------|------|----------|
| 15 | [`port`](../param/15_port.md) | Port | .serve |
| 16 | [`bind`](../param/16_bind.md) | String | .serve |
| 17 | [`open`](../param/17_open.md) | Boolean | .serve |
| 21 | [`journal_dir`](../param/21_journal_dir.md) | Path | all |
| 24 | [`no_color`](../param/24_no_color.md) | Boolean | all |
| 27 | [`refresh`](../param/27_refresh.md) | Integer | .serve |

### Interaction Rules

- `journal_dir` and `no_color` are accepted by **every** command, including the ones that write rather than read (`.prune`, `.export`, `.chart`) — a global param that worked on only some commands would be a trap
- `journal_dir` resolution: CLI param > `CLR_JOURNAL_DIR` env > `~/.clr/journal/` default
- `journal_dir` selects the journal *location*; the event-directory filter is [`dir`](../param/07_dir.md), a distinct Filtering-group param. They may be combined
- `no_color` is also triggered by the `NO_COLOR` env var; either input alone suppresses color
- `port`, `bind`, `open`, `refresh` are serve-specific but grouped here as infrastructure params
- `no_color` has no effect when `format` is not `table` (json/jsonl/csv have no color codes)
- Any `key::value` outside a command's accepted set is a hard error (exit 1) naming the offending key and listing what is accepted — an unread filter would otherwise widen output silently

### Commands

| # | Command | Available Members |
|---|---------|-------------------|
| 1 | [`.list`](../command/01_list.md) | journal_dir, no_color |
| 2 | [`.tail`](../command/02_tail.md) | journal_dir, no_color |
| 3 | [`.stats`](../command/03_stats.md) | journal_dir, no_color |
| 4 | [`.search`](../command/04_search.md) | journal_dir, no_color |
| 5 | [`.serve`](../command/05_serve.md) | journal_dir, no_color, port, bind, open, refresh |
| 6 | [`.prune`](../command/06_prune.md) | journal_dir, no_color |
| 7 | [`.status`](../command/07_status.md) | journal_dir, no_color |
| 8 | [`.export`](../command/08_export.md) | journal_dir, no_color |
| 9 | [`.chart`](../command/09_chart.md) | journal_dir, no_color |

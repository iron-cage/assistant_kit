# Parameter Group: Global

Cross-command parameters and serve-specific configuration.

### Members

| # | Parameter | Type | Commands |
|---|-----------|------|----------|
| 15 | [`port`](../param/15_port.md) | Port | .serve |
| 16 | [`bind`](../param/16_bind.md) | String | .serve |
| 17 | [`open`](../param/17_open.md) | Boolean | .serve |
| 21 | [`journal_dir`](../param/21_journal_dir.md) | Path | .list, .tail, .search, .stats, .status |
| 24 | [`no_color`](../param/24_no_color.md) | Boolean | .list, .tail, .stats |
| 27 | [`refresh`](../param/27_refresh.md) | Integer | .serve |

### Interaction Rules

- `journal_dir` applies to all read commands; resolution: CLI param > `CLR_JOURNAL_DIR` env > `~/.clr/journal/` default
- `no_color` applies to all table-rendering commands; also triggered by `NO_COLOR` env var
- `port`, `bind`, `open`, `refresh` are serve-specific but grouped here as infrastructure params
- `no_color` has no effect when `format` is not `table` (json/jsonl/csv have no color codes)

### Commands

| # | Command | Available Members |
|---|---------|-------------------|
| 1 | [`.list`](../command/01_list.md) | journal_dir, no_color |
| 2 | [`.tail`](../command/02_tail.md) | journal_dir, no_color |
| 3 | [`.stats`](../command/03_stats.md) | journal_dir, no_color |
| 4 | [`.search`](../command/04_search.md) | journal_dir |
| 5 | [`.serve`](../command/05_serve.md) | port, bind, open, refresh |
| 7 | [`.status`](../command/07_status.md) | journal_dir |

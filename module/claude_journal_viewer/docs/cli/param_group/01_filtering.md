# Parameter Group: Filtering

Time window, event type, and field-match filter parameters.
All filtering params are optional and combine with AND semantics —
an event must match ALL specified filters.

### Members

| # | Parameter | Type | Commands |
|---|-----------|------|----------|
| 01 | [`since`](../param/01_since.md) | Duration | .list, .search, .stats, .export |
| 02 | [`until`](../param/02_until.md) | Duration | .list, .search, .stats, .export |
| 03 | [`type`](../param/03_type.md) | EventType | .list, .tail, .search, .stats, .export |
| 04 | [`command`](../param/04_command.md) | String | .list, .tail, .search, .export |
| 05 | [`exit`](../param/05_exit.md) | Integer | .list |
| 06 | [`model`](../param/06_model.md) | String | .list |
| 07 | [`dir`](../param/07_dir.md) | Path | .list |
| 08 | [`creds`](../param/08_creds.md) | String | .list |

### Interaction Rules

- `since` and `until` define a time window: `[now - since, now - until]`
- When `since` is set but `until` is not: `[now - since, now]`
- When `until` is set but `since` is not: `[beginning, now - until]`
- `since` and `until` must not result in an empty window (since > until from now)
- All other filters (type, command, exit, model, dir, creds) are AND-combined
- An event missing a filtered field is excluded (e.g., filtering by `model` excludes events without a model field)

### Commands

| # | Command | Available Members |
|---|---------|-------------------|
| 1 | [`.list`](../command/01_list.md) | All 8 |
| 2 | [`.tail`](../command/02_tail.md) | type, command |
| 3 | [`.stats`](../command/03_stats.md) | since, until, type |
| 4 | [`.search`](../command/04_search.md) | since, type, command |
| 8 | [`.export`](../command/08_export.md) | since, until, type, command |

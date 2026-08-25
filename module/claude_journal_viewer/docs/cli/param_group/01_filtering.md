# Parameter Group: Filtering

Time window, event type, and field-match filter parameters.
All filtering params are optional and combine with AND semantics —
an event must match ALL specified filters.

### Members

| # | Parameter | Type | Commands |
|---|-----------|------|----------|
| 01 | [`since`](../param/01_since.md) | Duration | .list, .stats, .search, .export |
| 02 | [`until`](../param/02_until.md) | Duration | .list, .tail, .stats, .search, .export |
| 03 | [`type`](../param/03_type.md) | EventType | .list, .tail, .stats, .search, .export |
| 04 | [`command`](../param/04_command.md) | String | .list, .tail, .stats, .search, .export |
| 05 | [`exit`](../param/05_exit.md) | Integer | .list, .tail, .stats, .search, .export |
| 06 | [`model`](../param/06_model.md) | String | .list, .tail, .stats, .search, .export |
| 07 | [`dir`](../param/07_dir.md) | Path | .list, .tail, .stats, .search, .export |
| 08 | [`creds`](../param/08_creds.md) | String | .list, .tail, .stats, .search, .export |

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
| 2 | [`.tail`](../command/02_tail.md) | All except `since` |
| 3 | [`.stats`](../command/03_stats.md) | All 8 |
| 4 | [`.search`](../command/04_search.md) | All 8 |
| 8 | [`.export`](../command/08_export.md) | All 8 |

The five event-reading commands build the same `JournalFilter`, so they take the
same vocabulary. `.tail` is the one exception and drops exactly one member:
following the journal forward, there is no earlier event for `since::` to
exclude, so it is rejected rather than accepted-and-ignored. `until` survives
there with a weaker meaning — past the bound the follow goes quiet instead of
exiting.

Each command page enumerates a readable subset of this table rather than all
eight; the full accepted set is the one
[param/readme.md](../param/readme.md) lists per parameter.

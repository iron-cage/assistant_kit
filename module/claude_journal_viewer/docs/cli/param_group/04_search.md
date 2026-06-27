# Parameter Group: Search

Regex pattern and search scope parameters. Only used by `.search`.

### Members

| # | Parameter | Type | Commands |
|---|-----------|------|----------|
| 14 | [`pattern`](../param/14_pattern.md) | String | .search |
| 28 | [`include_stdout`](../param/28_include_stdout.md) | Boolean | .search |

### Interaction Rules

- `pattern` is required for `.search` — omitting it causes exit 1
- `include_stdout::0` (default): pattern is matched against the `message` field only
- `include_stdout::1`: pattern is also matched against `stdout` and `stderr` fields
- Searching stdout/stderr content is slower because these fields can be up to 1MB each (at `full` journal level)
- Events recorded at journal level `meta` have no stdout/stderr content — `include_stdout::1` matches nothing extra for those events

### Commands

| # | Command | Available Members |
|---|---------|-------------------|
| 4 | [`.search`](../command/04_search.md) | pattern, include_stdout |

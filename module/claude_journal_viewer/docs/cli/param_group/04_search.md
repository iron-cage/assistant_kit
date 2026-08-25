# Parameter Group: Search

Search-scope parameters. Only used by `.search`.

The group has one member. It had two until `include_stdout` was superseded —
`.search` matches `message`, `stdout`, `stderr`, `error_message`, `model`, and
`command` unconditionally, so there was no narrower default for a scope flag to
widen. Scope is now a property of the command, not a parameter, which is why
this group is a group of one rather than a pair.

### Members

| # | Parameter | Type | Commands |
|---|-----------|------|----------|
| 14 | [`pattern`](../param/14_pattern.md) | String | .search |

### Interaction Rules

- `pattern` is required for `.search` — omitting it causes exit 1
- `pattern` is matched with `str::contains`, not a regex engine — metacharacters are literal
- Searching is unconditional across `message`, `stdout`, `stderr`, `error_message`, `model`, and `command`; there is no parameter to narrow it, and [`include_stdout`](../param/28_include_stdout.md) is not accepted
- Searching stdout/stderr content is slower because these fields can be up to 1MB each (at `full` journal level)
- Events recorded at journal level `meta` carry no stdout/stderr, so for those events only `message`, `error_message`, `model`, and `command` can match — the prompt survives the level, its output does not
- The set is exactly those six: `dir`, `session_id`, and the other text fields are filterable or displayable, never matched against `pattern`
- `limit` is a Display-group member, not a Search one, and caps the events *searched* rather than the matches returned — see [`limit`](../param/09_limit.md)

### Commands

| # | Command | Available Members |
|---|---------|-------------------|
| 4 | [`.search`](../command/04_search.md) | pattern |

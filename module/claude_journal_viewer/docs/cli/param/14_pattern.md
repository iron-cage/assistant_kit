# CLI Parameter: pattern

Literal substring searched across event text. The value is matched with
`str::contains` — **not** compiled as a regex — case-sensitively, against each
event's `message`, `stdout`, `stderr`, `error_message`, `model`, and `command`
fields. An event matching in any one of them is kept.

- **Type:** [`String`](../type/03_string.md)
- **Default:** -- (none)
- **Required:** Yes (for `.search`)

```bash
clj .search pattern::"rate limit"              # Find rate limit events
clj .search pattern::"error" since::1d         # Errors in last day
clj .search pattern::"timeout" type::timeout   # Timeout events
clj .search pattern::"panic"                   # Panics anywhere in captured output
clj .search pattern::"refactor the parser"     # The prompt an event was given
```

Regex metacharacters are literal: `pattern::"(?i)panic"` looks for the eight
characters `(?i)pani…`, and finds nothing unless the output really contains
them. For case-insensitive or pattern matching, pipe `.list format::json`
through a tool that does it.

Filtering by *field* is separate from searching *text*: use
[`command::`](04_command.md) or [`model::`](06_model.md) to select on those
fields exactly, rather than relying on `pattern` to hit them incidentally. The
separation also runs the other way — [`dir::`](07_dir.md) filters on a field
`pattern` never reads, so a path that narrows the candidate set is not thereby
something `pattern` can match.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`String`](../type/03_string.md) | Fundamental | String | Any UTF-8 text; no pattern syntax, nothing to reject |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 4 | [Search](../param_group/04_search.md) | Full |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 4 | [`.search`](../command/04_search.md) | -- | Required parameter |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 2 | [Failure Diagnosis](../user_story/002_failure_diagnosis.md) | Developer |
| 3 | [Automation Audit](../user_story/003_automation_audit.md) | Developer |

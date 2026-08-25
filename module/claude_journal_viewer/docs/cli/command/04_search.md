# .search

Substring search across event text fields.

-- **Parameters:** pattern::, since::, until::, type::, command::, exit::, model::, dir::, creds::, limit::, journal_dir::, no_color::
-- **Exit Codes:** 0 (success, whether or not matches were found), 1 (missing `pattern`, or an invalid, unknown, or unimplemented param)

### Syntax

```
clj .search pattern::TEXT [since::DURATION] [until::DURATION] [type::EVENT_TYPE]
            [command::CMD] [exit::CODE] [model::NAME] [dir::SUBSTR] [creds::NAME]
            [limit::N] [journal_dir::PATH] [no_color::BOOL]
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `pattern` | String | -- | Yes | Literal substring to search for (case-sensitive; not a regex) |
| `since` | Duration | -- | No | Time window start |
| `type` | EventType | -- | No | Filter by event type |
| `command` | String | -- | No | Filter by clr command |
| `limit` | Integer | 50 | No | Max results |
| `journal_dir` | Path | ~/.clr/journal/ | No | Journal directory override |
| `no_color` | Boolean | 0 | No | Disable ANSI colors |

`.search` builds the same `JournalFilter` as `.list`, so `until`, `exit`,
`model`, `dir`, and `creds` are accepted here too.

`pattern` is matched with `str::contains`, not a regex engine — regex
metacharacters are literal. Searching is unconditional across `stdout`,
`stderr`, `error_message`, `model`, and `command`; there is no parameter to
widen or narrow that set (see [param/28_include_stdout.md](../param/28_include_stdout.md)).

An empty result set is exit **0** with `No events matching '<pattern>'.` — "found
nothing" is a valid answer, not a failure.

**Algorithm (3 steps):**

1. Read the required `pattern`; construct `JournalFilter` from the filter params
2. Query matching events, then keep those whose `stdout`, `stderr`, `error_message`, `model`, or `command` contains `pattern`
3. Render the survivors as a table, followed by an `N match(es)` footer

### Examples

```bash
clj .search pattern::"rate limit"               # Find rate limit events
clj .search pattern::"error" since::1d           # Errors in last day
clj .search pattern::"timeout" type::timeout     # Timeout events matching pattern
clj .search pattern::"Fix bug" include_stdout::1 # Search in stdout content
```

### Referenced User Stories

| # | User Story |
|---|-----------|
| 2 | [Failure Diagnosis](../user_story/002_failure_diagnosis.md) |
| 3 | [Automation Audit](../user_story/003_automation_audit.md) |

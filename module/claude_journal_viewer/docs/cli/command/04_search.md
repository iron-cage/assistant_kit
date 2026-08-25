# .search

Substring search across event text fields.

-- **Parameters:** pattern::, since::, until::, type::, command::, exit::, model::, dir::, creds::, limit::, journal_dir::, no_color::
-- **Exit Codes:** 0 (success, whether or not matches were found), 1 (missing `pattern`, or an invalid or unknown param)

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
| `limit` | Integer | -- | No | Cap on events searched (not on matches) |
| `journal_dir` | Path | ~/.clr/journal/ | No | Journal directory override |
| `no_color` | Boolean | 0 | No | Disable ANSI colors |

`.search` builds the same `JournalFilter` as `.list`, so `until`, `exit`,
`model`, `dir`, and `creds` are accepted here too. Unlike `.list`, no default
`limit` is applied: an unfiltered search reads the whole journal, and a `limit`
that *is* given caps the events searched before `pattern` runs — so
`limit::10` is "search the first 10 events", not "show the first 10 matches".

`pattern` is matched with `str::contains`, not a regex engine — regex
metacharacters are literal. Searching is unconditional across six fields —
`message`, `stdout`, `stderr`, `error_message`, `model`, `command` — and there
is no parameter to widen or narrow that set (see
[param/28_include_stdout.md](../param/28_include_stdout.md)).

`message` is the prompt the event was launched with, so a phrase you typed and
a phrase the model quoted back in its output are both reachable by the same
query:

```bash
clj .search pattern::"refactor the parser"   # matches the prompt and any echo of it
```

That field was excluded until the omission was fixed, and the exclusion was
invisible: a search for prompt text returned `No events matching '<pattern>'`,
which is exactly what `.search` says for a phrase genuinely absent from the
journal.

The set is exactly those six. Other text fields — `dir`, `session_id`,
`error_class`, and the rest — are filterable or displayable but never matched
against `pattern`. `dir::` narrows *which* events are considered; it never
decides whether `pattern` hit.

An empty result set is exit **0** with `No events matching '<pattern>'.` — "found
nothing" is a valid answer, not a failure.

**Algorithm (3 steps):**

1. Read the required `pattern`; construct `JournalFilter` from the filter params
2. Query matching events, then keep those whose `message`, `stdout`, `stderr`, `error_message`, `model`, or `command` contains `pattern`
3. Render the survivors as a table, followed by an `N match(es)` footer

### Examples

```bash
clj .search pattern::"rate limit"               # Find rate limit events
clj .search pattern::"error" since::1d           # Errors in last day
clj .search pattern::"timeout" type::timeout     # Timeout events matching pattern
clj .search pattern::"Fix bug"                   # stdout is searched without a flag
clj .search pattern::"refactor the parser"       # so is the prompt the event was given
```

### Referenced User Stories

| # | User Story |
|---|-----------|
| 2 | [Failure Diagnosis](../user_story/002_failure_diagnosis.md) |
| 3 | [Automation Audit](../user_story/003_automation_audit.md) |

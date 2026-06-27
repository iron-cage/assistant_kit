# .search

Full-text regex search across event data.

-- **Parameters:** pattern::, since::, type::, command::, limit::, include_stdout::, journal_dir::
-- **Exit Codes:** 0 (success, matches found), 1 (no matches or invalid param)

### Syntax

```
clj .search pattern::REGEX [since::DURATION] [type::EVENT_TYPE] [command::CMD]
            [limit::N] [include_stdout::BOOL] [journal_dir::PATH]
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `pattern` | String | -- | Yes | Regex pattern to search for |
| `since` | Duration | -- | No | Time window start |
| `type` | EventType | -- | No | Filter by event type |
| `command` | String | -- | No | Filter by clr command |
| `limit` | Integer | 50 | No | Max results |
| `include_stdout` | Boolean | 0 | No | Search in stdout content too |
| `journal_dir` | Path | ~/.clr/journal/ | No | Journal directory override |

**Algorithm (3 steps):**

1. Compile `pattern` as regex; construct `JournalFilter` from filter params
2. Iterate events, apply regex to `message` field (and `stdout` field if `include_stdout::1`)
3. Render matching events in table format with match context highlighted

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
| 2 | [Failure Diagnosis](../user_story/02_failure_diagnosis.md) |
| 3 | [Automation Audit](../user_story/03_automation_audit.md) |

# .export

Export filtered events to file in various formats.

-- **Parameters:** output::, format::, since::, until::, type::, command::, exit::, model::, dir::, creds::, limit::, journal_dir::, no_color::
-- **Exit Codes:** 0 (success), 1 (missing `output`, invalid or unknown param, or I/O error)

### Syntax

```
clj .export output::PATH [format::FORMAT] [since::DURATION] [until::DURATION]
            [type::EVENT_TYPE] [command::CMD] [exit::CODE] [model::NAME]
            [dir::SUBSTR] [creds::NAME] [limit::N] [journal_dir::PATH] [no_color::BOOL]
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `output` | Path | -- | **Yes** | Destination file |
| `format` | OutputFormat | json | No | Export format |
| `since` | Duration | -- | No | Time window start |
| `until` | Duration | -- | No | Time window end |
| `type` | EventType | -- | No | Filter by event type |
| `command` | String | -- | No | Filter by clr command |
| `journal_dir` | Path | ~/.clr/journal/ | No | Journal directory override |
| `no_color` | Boolean | 0 | No | Disable ANSI colors (`format::table` only) |

`.export` writes to a file; there is no stdout path. Omitting `output::` exits
1 with `Error: output:: parameter required`.

It builds the same `JournalFilter` as `.list`, so `exit`, `model`, `dir`,
`creds`, and `limit` are accepted here too. Unlike `.list`, no default `limit`
is applied — an unfiltered export writes the whole journal.

**Algorithm (3 steps):**

1. Read the required `output` path; construct filter from params and query all matching events
2. Serialize events in the selected format (default `json`)
3. Write to `output`, then print `Exported N event(s) to <path>`

### Examples

```bash
clj .export output::/tmp/week.csv format::csv since::7d      # CSV export
clj .export output::/tmp/month.json since::30d               # JSON (the default format)
clj .export output::/tmp/exec.jsonl format::jsonl type::execution
clj .export output::/tmp/day.txt format::table since::1d command::ask
```

### Referenced User Stories

| # | User Story |
|---|-----------|
| 3 | [Automation Audit](../user_story/003_automation_audit.md) |
| 4 | [Capacity Planning](../user_story/004_capacity_planning.md) |

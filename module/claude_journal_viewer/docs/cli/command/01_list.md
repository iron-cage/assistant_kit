# .list

List journal events with filtering and sort.

-- **Parameters:** since::, until::, type::, command::, exit::, model::, dir::, creds::, limit::, format::, sort::, reverse::
-- **Exit Codes:** 0 (success), 1 (invalid param)

### Syntax

```
clj .list [since::DURATION] [until::DURATION] [type::EVENT_TYPE] [command::CMD]
          [exit::CODE] [model::NAME] [dir::PATH] [creds::NAME] [limit::N]
          [format::FORMAT] [sort::FIELD] [reverse::BOOL]
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `since` | Duration | -- | No | Show events after this duration ago |
| `until` | Duration | -- | No | Show events before this duration ago |
| `type` | EventType | -- | No | Filter by event type |
| `command` | String | -- | No | Filter by clr command |
| `exit` | Integer | -- | No | Filter by exit code |
| `model` | String | -- | No | Filter by model name (substring) |
| `dir` | Path | -- | No | Filter by working directory (substring) |
| `creds` | String | -- | No | Filter by credential name |
| `limit` | Integer | 50 | No | Max events to display |
| `format` | OutputFormat | table | No | Output format |
| `sort` | SortField | time | No | Sort field |
| `reverse` | Boolean | 0 | No | Reverse sort order |

**Algorithm (3 steps):**

1. Construct `JournalFilter` from all filter params, open `JournalReader` at configured journal dir
2. Query events, apply sort + reverse, cap at `limit`
3. Render output in selected `format` (table/json/csv/jsonl)

### Examples

```bash
clj .list                                    # Last 50 events, table format
clj .list since::1h                          # Events from last hour
clj .list type::execution command::ask       # Only ask executions
clj .list since::7d sort::cost reverse::1    # Most expensive first, last 7 days
clj .list format::json limit::100            # JSON output, 100 events
clj .list exit::2 model::opus               # Rate-limit errors on opus model
```

### Referenced Parameter Groups

| Group | Excluded Params |
|-------|-----------------|
| [Filtering](../param_group/01_filtering.md) | -- |
| [Display](../param_group/02_display.md) | -- |
| [Global](../param_group/05_global.md) | -- |

### Referenced User Stories

| # | User Story |
|---|-----------|
| 1 | [Cost Tracking](../user_story/001_cost_tracking.md) |
| 2 | [Failure Diagnosis](../user_story/002_failure_diagnosis.md) |
| 4 | [Capacity Planning](../user_story/004_capacity_planning.md) |

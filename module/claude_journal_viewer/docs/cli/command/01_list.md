# .list

List journal events with filtering and sort.

-- **Parameters:** since::, until::, type::, command::, exit::, model::, dir::, creds::, limit::, format::, sort::, reverse::, journal_dir::, no_color::
-- **Exit Codes:** 0 (success), 1 (invalid, unknown, or unimplemented param)

### Syntax

```
clj .list [since::DURATION] [until::DURATION] [type::EVENT_TYPE] [command::CMD]
          [exit::CODE] [model::NAME] [dir::SUBSTR] [creds::NAME] [limit::N]
          [format::FORMAT] [sort::FIELD] [reverse::BOOL]
          [journal_dir::PATH] [no_color::BOOL]
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
| `dir` | Path | -- | No | Filter by the event's own working directory (substring) |
| `creds` | String | -- | No | Filter by credential name |
| `limit` | Integer | 50 | No | Max events to display; `0` = unlimited |
| `format` | OutputFormat | table | No | Output format |
| `sort` | SortField | time | No | Sort key |
| `reverse` | Boolean | 0 | No | Sort descending instead of ascending |
| `journal_dir` | Path | ~/.clr/journal/ | No | Journal directory override |
| `no_color` | Boolean | 0 | No | Disable ANSI colors |

`limit` caps the result *after* sorting, so `sort::cost reverse::1 limit::10`
returns the ten most expensive events across the whole matching window — not the
ten oldest re-ordered by cost.

Events missing the sort field sort below those that have it, so `reverse::1`
leads with real values and leaves the unknowns at the bottom. Ties keep journal
order in both directions.

**Not yet implemented:** `wide::`, `columns::`. These have parameter pages under
`docs/cli/param/` but no implementation; passing one exits 1 with a "not
implemented" message rather than being silently ignored.

### Unknown parameters

Any `key::value` outside the accepted set exits 1 naming the offending key and
listing what is accepted. This matters most for filters: a silently-ignored
filter *widens* the result set rather than erroring, so it reads as a query
that legitimately matched everything.

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

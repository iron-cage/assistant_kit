# .tail

Follow journal events in real-time.

-- **Parameters:** since::, until::, type::, command::, exit::, model::, dir::, creds::, limit::, no_color::, journal_dir::
-- **Exit Codes:** 0 (interrupted), 1 (invalid, unknown, or unimplemented param)

### Syntax

```
clj .tail [since::DURATION] [until::DURATION] [type::EVENT_TYPE] [command::CMD]
          [exit::CODE] [model::NAME] [dir::SUBSTR] [creds::NAME] [limit::N]
          [no_color::BOOL] [journal_dir::PATH]
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `type` | EventType | -- | No | Filter by event type |
| `command` | String | -- | No | Filter by clr command |
| `no_color` | Boolean | 0 | No | Disable ANSI colors |
| `journal_dir` | Path | ~/.clr/journal/ | No | Journal directory override |

`.tail` builds the same `JournalFilter` as `.list`, so it accepts the full
filter vocabulary — `since`, `until`, `exit`, `model`, `dir`, `creds`, `limit`
— not just the two listed above by name.

**Not yet implemented:** `format::`. Events are always rendered one per line in
table form; passing `format::` exits 1 rather than being silently ignored.

**Algorithm (3 steps):**

1. Open `JournalReader` at configured journal dir with filter from params
2. Call `JournalReader::tail()` which polls for new events at ~500ms intervals
3. For each new event, render one table-format line and flush stdout

### Examples

```bash
clj .tail                          # Follow all events
clj .tail type::execution         # Follow execution events only
clj .tail command::ask format::json  # Follow ask events as JSON
```

### Referenced User Stories

| # | User Story |
|---|-----------|
| 2 | [Failure Diagnosis](../user_story/002_failure_diagnosis.md) |
| 3 | [Automation Audit](../user_story/003_automation_audit.md) |

# .tail

Follow journal events in real-time.

-- **Parameters:** type::, command::, format::, no_color::, journal_dir::
-- **Exit Codes:** 0 (interrupted), 1 (invalid param)

### Syntax

```
clj .tail [type::EVENT_TYPE] [command::CMD] [format::FORMAT] [no_color::BOOL]
          [journal_dir::PATH]
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `type` | EventType | -- | No | Filter by event type |
| `command` | String | -- | No | Filter by clr command |
| `format` | OutputFormat | table | No | Output format per event |
| `no_color` | Boolean | 0 | No | Disable ANSI colors |
| `journal_dir` | Path | ~/.clr/journal/ | No | Journal directory override |

**Algorithm (3 steps):**

1. Open `JournalReader` at configured journal dir with filter from params
2. Call `JournalReader::tail()` which polls for new events at ~500ms intervals
3. For each new event, render one line in the selected format and flush stdout

### Examples

```bash
clj .tail                          # Follow all events
clj .tail type::execution         # Follow execution events only
clj .tail command::ask format::json  # Follow ask events as JSON
```

### Referenced User Stories

| # | User Story |
|---|-----------|
| 2 | [Failure Diagnosis](../user_story/02_failure_diagnosis.md) |
| 3 | [Automation Audit](../user_story/03_automation_audit.md) |

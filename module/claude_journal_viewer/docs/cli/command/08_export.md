# .export

Export filtered events to file in various formats.

-- **Parameters:** format::, since::, until::, type::, command::, output::
-- **Exit Codes:** 0 (success), 1 (invalid param or I/O error)

### Syntax

```
clj .export format::FORMAT [since::DURATION] [until::DURATION] [type::EVENT_TYPE]
            [command::CMD] [output::PATH]
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `format` | OutputFormat | jsonl | No | Export format |
| `since` | Duration | -- | No | Time window start |
| `until` | Duration | -- | No | Time window end |
| `type` | EventType | -- | No | Filter by event type |
| `command` | String | -- | No | Filter by clr command |
| `output` | Path | -- | No | Write to file instead of stdout |

**Algorithm (3 steps):**

1. Construct filter from params; query all matching events (no limit cap)
2. Serialize events in selected format
3. Write to `output` file path if provided, otherwise to stdout

### Examples

```bash
clj .export format::csv since::7d output::/tmp/week.csv    # CSV export
clj .export format::json since::30d                         # JSON to stdout
clj .export format::jsonl type::execution                   # Raw JSONL
clj .export format::table since::1d command::ask            # Table format
```

### Referenced User Stories

| # | User Story |
|---|-----------|
| 3 | [Automation Audit](../user_story/03_automation_audit.md) |
| 4 | [Capacity Planning](../user_story/04_capacity_planning.md) |

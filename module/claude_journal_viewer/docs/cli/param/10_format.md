# CLI Parameter: format

Output format for rendered events. Controls how events are
serialized to stdout. The `table` format renders a human-readable
aligned table; `json` emits a JSON array; `jsonl` emits one JSON
object per line; `csv` emits comma-separated values with a header row.

- **Type:** [`OutputFormat`](../type/06_output_format.md)
- **Default:** table (.list, .tail), json (.export)
- **Required:** No

```bash
clj .list format::json                # JSON array output
clj .list format::csv                 # CSV with header row
clj .tail format::json                # Follow events as JSON lines
clj .export format::csv output::/tmp/events.csv  # CSV file export
```

### `json` on `.tail`

`.tail` treats `json` and `jsonl` as the same format: one complete JSON object
per line. A JSON array cannot be produced by a stream that never ends — the
closing bracket would never be written, and a consumer piping to `jq` would
block forever waiting for it. Every other command renders `json` as a real
array.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`OutputFormat`](../type/06_output_format.md) | Enum | String | One of: table, json, jsonl, csv |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 2 | [Display](../param_group/02_display.md) | Full |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.list`](../command/01_list.md) | table | Human-readable table |
| 2 | [`.tail`](../command/02_tail.md) | table | One line per event; `json` == `jsonl` |
| 8 | [`.export`](../command/08_export.md) | json | JSON array written to `output::` |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 3 | [Automation Audit](../user_story/003_automation_audit.md) | Developer |
| 5 | [Team Reporting](../user_story/005_team_reporting.md) | Lead |

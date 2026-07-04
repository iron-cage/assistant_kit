# CLI Parameter: output

File path for `.export` output. When provided, serialized events
are written to the file instead of stdout. Parent directories
must exist — the command does not create them. Exit 1 on I/O error.

- **Type:** [`Path`](../type/05_path.md)
- **Default:** -- (stdout)
- **Required:** No

```bash
clj .export format::csv output::/tmp/week.csv since::7d  # CSV to file
clj .export format::json output::~/events.json            # JSON to file
clj .export format::jsonl                                  # JSONL to stdout
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`Path`](../type/05_path.md) | Semantic | String | Writable file path |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 2 | [Display](../param_group/02_display.md) | Partial (export only) |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 8 | [`.export`](../command/08_export.md) | -- | stdout when absent |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 3 | [Automation Audit](../user_story/003_automation_audit.md) | Developer |
| 5 | [Team Reporting](../user_story/005_team_reporting.md) | Lead |

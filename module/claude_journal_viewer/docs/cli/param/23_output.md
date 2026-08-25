# CLI Parameter: output

Destination file for `.export`. **Required** — `.export` has no stdout path, and
omitting `output::` exits 1 with `Error: output:: parameter required`. Parent
directories must exist; the command does not create them, and an I/O failure
exits 1.

Pipe-to-stdout is [`.list`](../command/01_list.md)'s job, not this one:
`clj .list format::jsonl` writes the same bytes `.export format::jsonl` would,
without a file. The one difference worth knowing is the cap — `.list` applies
its default `limit` of 50 and `.export` applies none, so the piped form needs an
explicit `limit::0` to match:

- **Type:** [`Path`](../type/05_path.md)
- **Default:** -- (none — the parameter is required)
- **Required:** **Yes**

```bash
clj .export output::/tmp/week.csv format::csv since::7d  # CSV to file
clj .export output::~/events.json                        # JSON (the default format)
clj .list format::jsonl limit::0 > /tmp/all.jsonl        # Same bytes, no file param
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
| 8 | [`.export`](../command/08_export.md) | -- | Required; exit 1 when absent |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 3 | [Automation Audit](../user_story/003_automation_audit.md) | Developer |
| 5 | [Team Reporting](../user_story/005_team_reporting.md) | Lead |

# CLI Parameter: include_stdout

Extend search scope to include stdout and stderr content fields.
By default, `.search` only matches against the `message` field.
When `1`, the regex pattern is also applied to `stdout` and
`stderr` fields, which contain the full subprocess output
captured at journal level `full`.

- **Type:** [`Boolean`](../type/08_boolean.md)
- **Default:** 0
- **Required:** No

```bash
clj .search pattern::"Fix bug" include_stdout::1   # Search in output
clj .search pattern::"panic" include_stdout::1      # Find panics in output
clj .search pattern::"Error" include_stdout::1 since::1d  # Errors in output today
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`Boolean`](../type/08_boolean.md) | Fundamental | Integer | 0 or 1 |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 4 | [Search](../param_group/04_search.md) | Full |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 4 | [`.search`](../command/04_search.md) | 0 | Message-only search |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 2 | [Failure Diagnosis](../user_story/02_failure_diagnosis.md) | Developer |
| 3 | [Automation Audit](../user_story/03_automation_audit.md) | Developer |

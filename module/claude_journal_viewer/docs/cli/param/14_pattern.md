# CLI Parameter: pattern

Regex pattern for full-text search across event data. The pattern
is compiled as a Rust regex and matched against the `message` field
of each event. When `include_stdout::1` is set, the pattern is also
matched against the `stdout` and `stderr` fields.

- **Type:** [`String`](../type/03_string.md)
- **Default:** -- (none)
- **Required:** Yes (for `.search`)

```bash
clj .search pattern::"rate limit"              # Find rate limit events
clj .search pattern::"error" since::1d         # Errors in last day
clj .search pattern::"timeout" type::timeout   # Timeout events
clj .search pattern::"Fix bug" include_stdout::1  # Search stdout
clj .search pattern::"(?i)panic"               # Case-insensitive search
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`String`](../type/03_string.md) | Fundamental | String | Valid Rust regex pattern |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 4 | [Search](../param_group/04_search.md) | Full |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 4 | [`.search`](../command/04_search.md) | -- | Required parameter |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 2 | [Failure Diagnosis](../user_story/02_failure_diagnosis.md) | Developer |
| 3 | [Automation Audit](../user_story/03_automation_audit.md) | Developer |

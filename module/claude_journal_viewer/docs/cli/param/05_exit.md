# CLI Parameter: exit

Filter events by subprocess exit code. Only events with a matching
`exit_code` field are included. Useful for isolating failures (non-zero)
or specific error classes (exit 2 = rate limit, exit 4 = timeout).

- **Type:** [`Integer`](../type/04_integer.md)
- **Default:** -- (all exit codes)
- **Required:** No

```bash
clj .list exit::0                     # Successful executions only
clj .list exit::2                     # Rate-limit failures
clj .list exit::4 since::1d          # Timeouts in last day
clj .list exit::1 command::ask       # Ask failures
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`Integer`](../type/04_integer.md) | Fundamental | Integer | Non-negative integer (0-255) |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 1 | [Filtering](../param_group/01_filtering.md) | Full |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.list`](../command/01_list.md) | -- | All exit codes |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 2 | [Failure Diagnosis](../user_story/02_failure_diagnosis.md) | Developer |

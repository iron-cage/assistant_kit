# CLI Parameter: include_stdout

**Superseded — not a parameter.** `.search` searches `stdout` and `stderr`
unconditionally, alongside `error_message`, `model`, and `command`. There is no
narrower default for this flag to widen, so the parameter carries no meaning
and is not accepted; passing it exits 1.

This page is retained because the surrounding documentation links to it. The
behavior it describes is the *current, unconditional* behavior of
[`.search`](../command/04_search.md) — search the output fields directly and
omit the flag.

- **Type:** -- (not accepted)
- **Default:** -- (stdout and stderr always searched)
- **Required:** No

```bash
clj .search pattern::"panic"              # Finds panics in stdout/stderr
clj .search pattern::"Error" since::1d    # Errors in output today
```

### Referenced Type

-- (none — the parameter is not accepted)

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 4 | [Search](../param_group/04_search.md) | Superseded |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 4 | [`.search`](../command/04_search.md) | -- | stdout/stderr always searched; flag not accepted |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 2 | [Failure Diagnosis](../user_story/002_failure_diagnosis.md) | Developer |
| 3 | [Automation Audit](../user_story/003_automation_audit.md) | Developer |

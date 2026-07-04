# CLI Parameter: command

Filter events by the originating CLR command name. Matches the
`command` field in the journal event record. Substring match
is NOT performed — the value must exactly match one of the
8 CLR commands: `run`, `ask`, `isolated`, `refresh`, `ps`, `kill`, `tools`, `help`.

- **Type:** [`String`](../type/03_string.md)
- **Default:** -- (all commands)
- **Required:** No

```bash
clj .list command::ask                # Only ask invocations
clj .list command::isolated           # Only credential-isolated runs
clj .stats command::run               # Stats for run command only
clj .search pattern::"timeout" command::ask  # Timeouts in ask mode
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`String`](../type/03_string.md) | Fundamental | String | Exact match against CLR command names |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 1 | [Filtering](../param_group/01_filtering.md) | Full |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.list`](../command/01_list.md) | -- | All commands |
| 2 | [`.tail`](../command/02_tail.md) | -- | All commands |
| 4 | [`.search`](../command/04_search.md) | -- | All commands |
| 8 | [`.export`](../command/08_export.md) | -- | All commands |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 3 | [Automation Audit](../user_story/003_automation_audit.md) | Developer |
| 5 | [Team Reporting](../user_story/005_team_reporting.md) | Lead |

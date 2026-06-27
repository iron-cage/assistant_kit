# CLI Parameter: columns

Comma-separated list of column names to display in table output.
Overrides the default column set. Available columns for `.list`:
`time`, `type`, `command`, `exit`, `model`, `duration`, `cost`,
`tokens_in`, `tokens_out`, `dir`, `creds`, `message`.

When not set, the default column set is:
`time,type,command,exit,duration,cost,model`

- **Type:** [`String`](../type/03_string.md)
- **Default:** -- (default column set)
- **Required:** No

```bash
clj .list columns::time,command,cost,exit       # Custom 4 columns
clj .list columns::time,model,tokens_in,tokens_out  # Token analysis
clj .list columns::time,dir,command,duration    # Per-project timing
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`String`](../type/03_string.md) | Fundamental | String | Comma-separated column names |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 2 | [Display](../param_group/02_display.md) | Full |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.list`](../command/01_list.md) | -- | Default column set |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Cost Tracking](../user_story/01_cost_tracking.md) | Developer |
| 5 | [Team Reporting](../user_story/05_team_reporting.md) | Lead |

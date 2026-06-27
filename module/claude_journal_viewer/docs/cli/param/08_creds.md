# CLI Parameter: creds

Filter events by the credential name used for the execution.
Exact match against the `creds` field in the event record.
Only `isolated` and `refresh` commands record a credential name.

- **Type:** [`String`](../type/03_string.md)
- **Default:** -- (all credentials)
- **Required:** No

```bash
clj .list creds::prod.json since::7d   # Events using prod creds
clj .list creds::staging.json          # Staging credential usage
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`String`](../type/03_string.md) | Fundamental | String | Exact match against creds field |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 1 | [Filtering](../param_group/01_filtering.md) | Full |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.list`](../command/01_list.md) | -- | All credentials |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 3 | [Automation Audit](../user_story/03_automation_audit.md) | Developer |

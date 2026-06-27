# CLI Parameter: dir

Filter events by the working directory recorded in the event.
Substring match — `/home/user/project` matches events from
`/home/user/project` and `/home/user/project/subdir`.

- **Type:** [`Path`](../type/05_path.md)
- **Default:** -- (all directories)
- **Required:** No

```bash
clj .list dir::/home/user/myproject   # Events from specific project
clj .list dir::myproject since::7d   # Substring match, last week
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`Path`](../type/05_path.md) | Semantic | String | Substring match against dir field |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 1 | [Filtering](../param_group/01_filtering.md) | Full |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.list`](../command/01_list.md) | -- | All directories |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 3 | [Automation Audit](../user_story/03_automation_audit.md) | Developer |

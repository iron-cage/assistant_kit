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

`dir::` never changes *where the journal is read from* — that is
[`journal_dir::`](21_journal_dir.md), a separate global parameter. The two are
independent and may be combined: `clj .list journal_dir::/var/log/clr
dir::/work/alpha` reads the alternate journal and shows only the events whose
own working directory contains `/work/alpha`.

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
| 2 | [`.tail`](../command/02_tail.md) | -- | All directories |
| 3 | [`.stats`](../command/03_stats.md) | -- | All directories |
| 4 | [`.search`](../command/04_search.md) | -- | All directories |
| 8 | [`.export`](../command/08_export.md) | -- | All directories |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 3 | [Automation Audit](../user_story/003_automation_audit.md) | Developer |

# CLI Parameter: journal_dir

Override the journal directory path. Defaults to `~/.clr/journal/`
which is the canonical `CLR_JOURNAL_DIR` location shared with `clr`.
Useful for inspecting journal data from other users or non-default
locations.

- **Type:** [`Path`](../type/05_path.md)
- **Default:** ~/.clr/journal/ (or `CLR_JOURNAL_DIR` env var)
- **Required:** No

```bash
clj .list journal_dir::/tmp/test_journal   # Custom journal path
clj .status journal_dir::/var/log/clr      # Status of alternate journal
clj .tail journal_dir::~/other/.clr/journal/  # Follow alternate journal
```

The resolution order is:
1. `journal_dir::` CLI parameter (highest priority)
2. `CLR_JOURNAL_DIR` environment variable
3. `~/.clr/journal/` default

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`Path`](../type/05_path.md) | Semantic | String | Existing directory path |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 5 | [Global](../param_group/05_global.md) | Full |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.list`](../command/01_list.md) | ~/.clr/journal/ | Standard location |
| 2 | [`.tail`](../command/02_tail.md) | ~/.clr/journal/ | Standard location |
| 3 | [`.stats`](../command/03_stats.md) | ~/.clr/journal/ | Standard location |
| 4 | [`.search`](../command/04_search.md) | ~/.clr/journal/ | Standard location |
| 7 | [`.status`](../command/07_status.md) | ~/.clr/journal/ | Standard location |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 5 | [Team Reporting](../user_story/005_team_reporting.md) | Lead |

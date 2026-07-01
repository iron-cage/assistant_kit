# CLI Parameter: --journal-dir

Override the journal directory path. The journal writer creates
daily JSONL files in this directory. Defaults to `~/.clr/journal/`.
The directory is created if it does not exist.

- **Type:** `Path`
- **Default:** ~/.clr/journal/ (or `CLR_JOURNAL_DIR` env var)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md), [`isolated`](../command/02_isolated.md), [`refresh`](../command/03_refresh.md)
- **JSON Key:** `"journal-dir"`

```sh
clr "test" --journal-dir /tmp/test_journal   # Custom path
clr "test" --journal-dir ~/alt/.clr/journal  # Alternate home
CLR_JOURNAL_DIR=/var/log/clr clr "test"      # Via env var
```

Resolution order:
1. `--journal-dir` CLI flag (highest priority)
2. `CLR_JOURNAL_DIR` environment variable
3. `~/.clr/journal/` default

**Note:** This is the same `CLR_JOURNAL_DIR` env var used by `clj`
(the journal viewer). Both writer and reader share one canonical
env var for journal location.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Path | Semantic | String | Directory path (created if absent) |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | --journal |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | ~/.clr/journal/ | Standard location |
| 2 | [`isolated`](../command/02_isolated.md) | ~/.clr/journal/ | Standard location |
| 3 | [`refresh`](../command/03_refresh.md) | ~/.clr/journal/ | Standard location |
| 5 | [`ask`](../command/05_ask.md) | ~/.clr/journal/ | Standard location |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 18 | [018_env_var_configuration.md](../user_story/018_env_var_configuration.md) | Developer |

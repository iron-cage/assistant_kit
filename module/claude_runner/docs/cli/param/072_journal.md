# CLI Parameter: --journal

Journal recording level. Controls how much data is captured
per event in the journal file. Default is `full` — all event
fields including complete stdout and stderr (truncated at 1 MB).

- **Type:** `JournalLevel` (enum: `full`, `meta`, `off`)
- **Default:** full
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md), [`isolated`](../command/03_isolated.md), [`refresh`](../command/04_refresh.md)
- **JSON Key:** `"journal"`

```sh
clr "test" --journal full    # Full output captured (default)
clr "test" --journal meta    # Metadata only (no stdout/stderr)
clr "test" --journal off     # No journaling
```

**Note:** `full` is the default to maximize diagnostic value.
The `meta` level is useful when stdout/stderr contains large binary
output or sensitive data that should not be journaled. `off` disables
journaling entirely.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| JournalLevel | Enum | String | One of: full, meta, off |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | --journal-dir |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | full | Print and interactive mode |
| 2 | [`isolated`](../command/03_isolated.md) | full | Credential-isolated execution |
| 3 | [`refresh`](../command/04_refresh.md) | full | Credential refresh |
| 5 | [`ask`](../command/05_ask.md) | full | Ask mode |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 18 | [018_env_var_configuration.md](../user_story/018_env_var_configuration.md) | Developer |

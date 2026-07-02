# CLI Parameter: --no-compact-window

Suppress the default `CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000` injection. By default all four
running commands (`run`, `ask`, `isolated`, `refresh`) inject this variable to enable context-window
auto-compaction in the Claude subprocess. Use `--no-compact-window` when the caller's environment
already sets `CLAUDE_CODE_AUTO_COMPACT_WINDOW` to a different value, or when compaction is
undesirable for a specific invocation.

- **Type:** bool
- **Default:** false
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md), [`isolated`](../command/03_isolated.md), [`refresh`](../command/04_refresh.md)
- **Group:** [Running Commands](../param_group/06_running_commands.md)
- **Env:** `CLR_NO_COMPACT_WINDOW` (bool: `"1"` or `"true"`; applied only when `--no-compact-window` absent from CLI)

```sh
clr --no-compact-window "my task"        # suppress CLAUDE_CODE_AUTO_COMPACT_WINDOW injection
clr isolated --no-compact-window "task"  # same opt-out for isolated
CLR_NO_COMPACT_WINDOW=1 clr "task"      # env var fallback
```

**Effect:** When set, `CLAUDE_CODE_AUTO_COMPACT_WINDOW` is omitted from the subprocess
environment entirely. The Claude subprocess uses its own built-in default or whatever value the
caller's environment already provides.

**Injected var details:** Without `--no-compact-window`, the runner sets
`CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000` in the subprocess environment for every running
command. This enables automatic context-window compaction at the 200,000-token mark, preventing
out-of-context failures in long-running tasks. The `--dry-run` / `--trace` output reflects the
resolved value (or absence) accurately.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| bool | Primitive | bool | present/absent |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 6 | [Running Commands](../param_group/06_running_commands.md) | Full | 5 other universal params |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | false | — |
| 2 | [`isolated`](../command/03_isolated.md) | false | — |
| 3 | [`refresh`](../command/04_refresh.md) | false | — |
| 5 | [`ask`](../command/05_ask.md) | false | — |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 18 | [018_env_var_configuration.md](../user_story/018_env_var_configuration.md) | Developer |

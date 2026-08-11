# CLI Parameter: --no-effort-max

Suppress the automatic `--effort max` injection. When set, no `--effort` flag
is forwarded to the `claude` subprocess at all.

- **Type:** bool (standalone flag)
- **Default:** false (effort max injection is **ON** by default; this flag turns it **OFF**)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **JSON Key:** `"no-effort-max"`

```sh
clr "Fix bug"                      # sends: --effort max (default)
clr --no-effort-max "Fix bug"      # sends: no --effort flag at all
```

**Note:** Use `--no-effort-max` when targeting models or configurations that
do not support the `--effort` flag, or when you need claude's native default
effort level without any override.

**Interactive mode workaround (BUG-434):** Claude Code v2.1.78+ rejects `"max"` as an
effort level in interactive mode. Until TSK-447 lands a mode-aware injection fix, use
`--no-effort-max` (or `--effort high`) on bare `clr` interactive invocations to avoid the
"Effort level 'max' is not available in interactive mode" error.

**Note:** `--effort <level>` and `--no-effort-max` are mutually exclusive.
If `--no-effort-max` is set, any `--effort` value is ignored.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| bool | Primitive | bool | present/absent |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | 16 other params |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | false | — |
| 5 | [`ask`](../command/05_ask.md) | false | — |
| 3 | [`isolated`](../command/03_isolated.md) | false | Suppresses automatic `--effort max` injection; env: `CLR_NO_EFFORT_MAX` |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 20 | [020_suppress_effort_max.md](../user_story/020_suppress_effort_max.md) | Developer |

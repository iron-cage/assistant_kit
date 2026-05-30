# CLI Parameter: --keep-claudecode

Opt-out flag that preserves the `CLAUDECODE` environment variable in the
`claude` subprocess environment. Default behaviour (without this flag) is to
remove `CLAUDECODE` before spawning, enabling clean nested invocations from
within a Claude Code session.

- **Type:** bool
- **Default:** false (CLAUDECODE is removed — subprocess runs as standalone)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)

```sh
clr "Fix bug"                    # CLAUDECODE removed (default — standalone behaviour)
clr --keep-claudecode "Fix bug"  # CLAUDECODE preserved — subprocess sees nested env
```

**Background:** When `clr` is invoked from within a Claude Code session, the
parent session sets `CLAUDECODE=1` in its environment. A child `claude`
process that inherits this variable treats itself as a nested agent, which
alters permission handling, output format, and tool availability. Removing
`CLAUDECODE` before spawning causes the subprocess to behave as a first-class
standalone Claude Code process — the correct default for automation.

**Note:** Use `--keep-claudecode` only when you specifically want the
subprocess to operate in nested-agent mode. This is rare; the default covers
virtually all automation use-cases.

**Note:** `--keep-claudecode` has no effect when the parent process does not
have `CLAUDECODE` set — it is a no-op in that environment.

**Env var:** `CLR_KEEP_CLAUDECODE` — accepts `1` or `true` (case-insensitive); applied when
`--keep-claudecode` is absent from the CLI. `CLAUDECODE=1 CLR_KEEP_CLAUDECODE=1 clr "task"`
is equivalent to `CLAUDECODE=1 clr --keep-claudecode "task"`.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| bool | Primitive | bool | present/absent |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | 15 other params |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | false | — |
| 5 | [`ask`](../command/05_ask.md) | false | — |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 21 | [021_keep_claudecode_context.md](../user_story/021_keep_claudecode_context.md) | Developer |

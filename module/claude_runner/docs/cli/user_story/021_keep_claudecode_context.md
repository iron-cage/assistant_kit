# CLI User Story: Keep ClaudeCode Context

### Scope

- **Purpose**: Document the `--keep-claudecode` opt-out flag that preserves CLAUDECODE in the subprocess env.
- **Responsibility**: Define acceptance criteria for the nested-agent mode opt-in via env var preservation.
- **In Scope**: Default CLAUDECODE removal, `--keep-claudecode` preservation, `CLR_KEEP_CLAUDECODE` env var, no-op without parent CLAUDECODE.
- **Out of Scope**: Claude Code nested-agent mode internals, permission handling in nested context.

### Persona

Developer running `clr` from within a Claude Code session who specifically wants the subprocess to operate in nested-agent mode (rare use case; the default covers virtually all automation needs).

### Goal

Opt into nested-agent subprocess behavior by preserving the `CLAUDECODE` environment variable, overriding the default behavior that removes it to ensure clean standalone execution.

### Acceptance Criteria

- `clr "Task"` removes `CLAUDECODE` from the subprocess environment by default (standalone mode)
- `clr --keep-claudecode "Task"` preserves the parent's `CLAUDECODE` value in the subprocess
- `CLR_KEEP_CLAUDECODE=1 clr "Task"` applies preservation via env var
- `--keep-claudecode` is a no-op when the parent process has no `CLAUDECODE` set

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`run`](../command/01_run.md) | Default command; CLAUDECODE removed by default |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | `--keep-claudecode` is a runner control flag |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 27 | [`--keep-claudecode`](../param/027_keep_claudecode.md) | Preserve CLAUDECODE env var in subprocess |

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 18 | [Env-var Configuration](018_env_var_configuration.md) | `CLR_KEEP_CLAUDECODE` is one of 25 CLR_* env vars |

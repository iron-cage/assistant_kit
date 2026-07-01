# Preserve CLAUDECODE for nested-agent subprocess behavior

**Persona:** Developer running `clr` from within a Claude Code session who specifically wants the subprocess to operate in nested-agent mode (rare use case; the default covers virtually all automation needs).
**Goal:** Opt into nested-agent subprocess behavior by preserving the `CLAUDECODE` environment variable, overriding the default behavior that removes it to ensure clean standalone execution.
**Benefit:** Enables nested-agent mode for advanced use cases where the subprocess must behave as a Claude Code plugin.
**Priority:** Low

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

### Workflow Steps

1. `clr --keep-claudecode "Task"` — preserve the parent's `CLAUDECODE` value in the subprocess
2. `CLR_KEEP_CLAUDECODE=1 clr "Task"` — apply preservation via environment variable
3. `clr --keep-claudecode --dry-run "Task"` — verify `CLAUDECODE` is preserved in the assembled env block

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 18 | [Env-var Configuration](018_env_var_configuration.md) | `CLR_KEEP_CLAUDECODE` is one of 25 CLR_* env vars |

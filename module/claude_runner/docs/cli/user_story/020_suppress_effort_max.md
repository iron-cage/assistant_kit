# Suppress automatic effort-max injection

**Persona:** Developer targeting models or configurations that do not support the `--effort` flag, or who needs Claude's native default effort level without any override.
**Goal:** Suppress the automatic `--effort max` injection to run Claude with no effort override, either for compatibility with models that don't support the flag or when native defaults are preferred.
**Benefit:** Enables compatibility with models that do not support the effort flag and access to Claude's native effort defaults.
**Priority:** Low

### Acceptance Criteria

- `clr "Task"` includes `--effort max` in the assembled command by default
- `clr --no-effort-max "Task"` omits all `--effort` flags from the assembled command
- `--no-effort-max` wins over any explicit `--effort` flag when both are present
- `CLR_NO_EFFORT_MAX=1 clr "Task"` applies suppression via env var

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`run`](../command/01_run.md) | Default command; effort max is injected by default |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | `--no-effort-max` is a runner control flag |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 17 | [`--effort`](../param/017_effort.md) | Effort level; default max is suppressed by `--no-effort-max` |
| 18 | [`--no-effort-max`](../param/018_no_effort_max.md) | Suppress automatic effort max injection |

### Workflow Steps

1. `clr --no-effort-max "Task"` — run without automatic `--effort max` injection
2. `CLR_NO_EFFORT_MAX=1 clr "Task"` — suppress effort max via environment variable
3. `clr --no-effort-max --dry-run "Task"` — verify that `--effort` is absent from the assembled command

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 15 | [Ask Mode](015_ask_mode.md) | Ask is a pure alias for run; uses same `--effort max` default |
| 18 | [Env-var Configuration](018_env_var_configuration.md) | `CLR_NO_EFFORT_MAX` is one of 25 CLR_* env vars |

# CLI User Story: Suppress Effort Max

### Scope

- **Purpose**: Document the `--no-effort-max` flag that suppresses automatic `--effort max` injection.
- **Responsibility**: Define acceptance criteria for suppression behavior, flag precedence, and env var fallback.
- **In Scope**: Default `--effort max` injection, `--no-effort-max` suppression, precedence over `--effort`, `CLR_NO_EFFORT_MAX` env var.
- **Out of Scope**: Effort level selection (→ `--effort` param), ask mode effort defaults (→ 015_ask_mode.md).

### Persona

Developer targeting models or configurations that do not support the `--effort` flag, or who needs Claude's native default effort level without any override.

### Goal

Suppress the automatic `--effort max` injection to run Claude with no effort override, either for compatibility with models that don't support the flag or when native defaults are preferred.

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

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 15 | [Ask Mode](015_ask_mode.md) | Ask mode uses `--effort high` default instead of max |
| 18 | [Env-var Configuration](018_env_var_configuration.md) | `CLR_NO_EFFORT_MAX` is one of 25 CLR_* env vars |

# CLI User Story: Model Selection

### Scope

- **Purpose**: Document model selection via `--model` and `CLR_MODEL` for choosing the Claude model.
- **Responsibility**: Define acceptance criteria for explicit model override at the invocation level.
- **In Scope**: `--model` flag forwarding, `CLR_MODEL` env var fallback, CLI precedence, ask command support.
- **Out of Scope**: Effort-level tuning (→ 006_verbose_debugging.md), ask defaults (→ 015_ask_mode.md).

### Persona

Developer who needs a specific Claude model — such as a faster model for simple tasks or a more capable one for complex analysis — and wants to override the default model for a single invocation.

### Goal

Override the Claude model for a single invocation by passing `--model` or setting `CLR_MODEL`, with the CLI flag always winning when both are present.

### Acceptance Criteria

- `clr --model sonnet "Task"` forwards `--model sonnet` to the `claude` subprocess
- `CLR_MODEL=haiku clr "Task"` applies the env var when `--model` is absent
- An explicit `--model` CLI flag overrides a concurrent `CLR_MODEL` env var
- `clr ask --model sonnet "Question"` accepts `--model` in ask mode as well

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`run`](../command/01_run.md) | Default command; `--model` selects the model |
| 5 | [`ask`](../command/05_ask.md) | Ask command also accepts `--model` |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | `--model` is a Claude-native flag |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 3 | [`--model`](../param/003_model.md) | Model name forwarded to claude subprocess |

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 18 | [Env-var Configuration](018_env_var_configuration.md) | `CLR_MODEL` is one of 25 CLR_* env vars |

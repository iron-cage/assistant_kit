# CLI Parameter: --max-tokens

Set the maximum number of output tokens for the Claude Code subprocess.
Passed via the `CLAUDE_CODE_MAX_OUTPUT_TOKENS` environment variable.

- **Type:** [`TokenLimit`](../type/03_token_limit.md)
- **Default:** 200000
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **Validation:** must be a valid u32 (0–4294967295); non-numeric → error

```sh
clr "Summarize" --max-tokens 50000
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`TokenLimit`](../type/03_token_limit.md) | Semantic | unsigned 32-bit integer | 0 to 4294967295 |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | 16 other params |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | 200000 | — |
| 5 | [`ask`](../command/05_ask.md) | 200000 | Pure alias — same default as `run` |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 15 | [015_ask_mode.md](../user_story/015_ask_mode.md) | Developer |

# CLI Type: SystemPromptText

Free-form text that sets or extends the system prompt sent to the claude
subprocess. Semantically distinct from `MessageText` (Type 1): this is the
model's behavioral context (system turn), not the user's conversational input
(user turn).

- **Purpose:** Free-form system prompt text (system turn, not user turn)
- **Fundamental Type:** String
- **Constants:** —
- **Constraints:** any UTF-8 text; no length limit
- **Parsing:** consumed as the next token after `--system-prompt` or `--append-system-prompt`
- **Methods:** —

```sh
clr --system-prompt "You are a Rust expert. Be concise." "Review PR"
clr --append-system-prompt "Always respond in JSON." "List failing tests"
```

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 1 | [`run`](../command/01_run.md) | `--system-prompt`, `--append-system-prompt` |
| 5 | [`ask`](../command/05_ask.md) | `--system-prompt`, `--append-system-prompt` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 15 | [`--system-prompt`](../param/015_system_prompt.md) | 2 |
| 16 | [`--append-system-prompt`](../param/016_append_system_prompt.md) | 2 |

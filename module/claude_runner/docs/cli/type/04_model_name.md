# CLI Type: ModelName

Identifier for a Claude model variant. Passed through to the claude
subprocess via `--model`.

- **Purpose:** Claude model identifier string
- **Fundamental Type:** String
- **Constants:** `DEFAULT_MODEL` = `claude-opus-5-5` (built-in default — see [`--model`](../param/003_model.md)); `REFRESH_DEFAULT_MODEL` = `claude-sonnet-5` (`refresh` only)
- **Constraints:** any non-empty string accepted by `claude --model`
- **Parsing:** consumed as the next token after `--model`
- **Methods:** —

```sh
clr --model sonnet -p "Explain"
clr --model opus "Fix bug"
```

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 1 | [`run`](../command/01_run.md) | `--model` |
| 5 | [`ask`](../command/05_ask.md) | `--model` |
| 11 | [`topic`](../command/11_topic.md) | `--model` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 3 | [`--model`](../param/003_model.md) | 3 |

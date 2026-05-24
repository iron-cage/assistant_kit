# CLI Type: TokenLimit

Maximum number of output tokens for the Claude Code subprocess. Set via
the `CLAUDE_CODE_MAX_OUTPUT_TOKENS` environment variable.

- **Purpose:** Maximum output token count
- **Fundamental Type:** unsigned 32-bit integer
- **Constants:** —
- **Constraints:** 0 to 4294967295; default 200000
- **Parsing:** integer parse; rejects negative, float, non-numeric
- **Methods:** —

```sh
# Valid
clr --max-tokens 0 "test"            # minimum
clr --max-tokens 4294967295 "test"   # maximum

# Invalid
clr --max-tokens -1 "test"           # negative → error
clr --max-tokens 4294967296 "test"   # overflow → error
clr --max-tokens abc "test"          # non-numeric → error
```

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 1 | [`run`](../command/01_run.md) | `--max-tokens` |
| 5 | [`ask`](../command/05_ask.md) | `--max-tokens` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 9 | [`--max-tokens`](../param/009_max_tokens.md) | 2 |

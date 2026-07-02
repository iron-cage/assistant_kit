# CLI Type: MessageText

Free-form prompt text sent to Claude Code. Multiple positional words in
argv are joined with a single space.

- **Purpose:** Free-form prompt text sent to the `claude` subprocess
- **Fundamental Type:** String
- **Constants:** —
- **Constraints:** any UTF-8 text; no length limit
- **Parsing:** all non-flag tokens collected, joined with `" "`
- **Methods:** —

```sh
clr "Fix the auth bug"      # single-token message
clr Fix the auth bug        # multi-token → "Fix the auth bug"
clr -- --not-a-flag         # after --, everything is positional
```

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 1 | [`run`](../command/01_run.md) | `[MESSAGE]` |
| 2 | [`isolated`](../command/03_isolated.md) | `[MESSAGE]` |
| 5 | [`ask`](../command/05_ask.md) | `[MESSAGE]` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 1 | [`[MESSAGE]`](../param/001_message.md) | 3 |

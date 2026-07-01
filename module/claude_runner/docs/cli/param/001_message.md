# CLI Parameter: [MESSAGE]

Free-form prompt text sent to Claude Code. Multiple positional words are
joined with spaces. When a message is given, print mode is the default;
use `--interactive` to override to TTY passthrough.

- **Type:** [`MessageText`](../type/01_message_text.md)
- **Default:** — (none; interactive REPL when absent)
- **Command:** [`run`](../command/01_run.md)
- **JSON Key:** `"message"`

```sh
clr "Fix the bug in auth.rs"
clr Fix the bug       # equivalent — words joined with space
```

### Shell Quoting

The unquoted multi-word form works when no word contains shell-special characters. When the
message includes `(`, `)`, `&`, `;`, `|`, `` ` ``, `$`, `!`, `{`, `}`, `<`, or `>`, bash
tokenizes them before `clr` is invoked — the command fails with a shell syntax error.

Use any of these alternatives:

```sh
clr "Fix (the nasty bug)"            # double-quote the whole message
clr 'Fix (the nasty bug)'            # single-quote suppresses all expansion

CLR_MESSAGE='Fix (a nasty bug)' clr  # env var: no quoting needed; bypasses shell parsing
```

`CLR_MESSAGE` is the recommended escape hatch for messages with shell-special characters —
it bypasses bash argument parsing entirely. The env var is applied when no positional message
is present on the CLI.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`MessageText`](../type/01_message_text.md) | Semantic | String | any UTF-8 text |

### Referenced Parameter Groups

*None — `[MESSAGE]` is a positional argument, not a member of any parameter group.*

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | — | Triggers print mode when present |
| 2 | [`isolated`](../command/02_isolated.md) | — | Forwarded to claude subprocess |
| 5 | [`ask`](../command/05_ask.md) | — | Always print mode |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 2 | [002_print_mode_capture.md](../user_story/002_print_mode_capture.md) | Developer |
| 3 | [003_interactive_with_message.md](../user_story/003_interactive_with_message.md) | Developer |
| 11 | [011_file_input.md](../user_story/011_file_input.md) | Developer |
| 15 | [015_ask_mode.md](../user_story/015_ask_mode.md) | Developer |

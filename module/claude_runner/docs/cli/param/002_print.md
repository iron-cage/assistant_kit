# CLI Parameter: --print

Explicit print mode flag. When a message is given, print mode is the
default — this flag is a backward-compatible explicit alias.
Captures Claude's stdout and prints it instead of passing through
the TTY.

- **Aliases:** `-p`
- **Type:** bool (standalone flag)
- **Default:** auto (active when message given; inactive for bare REPL)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Claude-Native Flags](../param_group/01_claude_native_flags.md)
- **JSON Key:** `"print"`

```sh
clr "Explain this function"        # print mode by default
clr -p "Explain this function"     # same — explicit alias
output=$(clr "List files" --model sonnet)
```

**Note:** Print mode without a message exits with error code 1.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| bool | Primitive | bool | present/absent |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | Full | `--model`, `--verbose`, `--effort`, `--json-schema`, `--mcp-config` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | auto | Default on when message given |
| 5 | [`ask`](../command/05_ask.md) | true | Always on for ask |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 2 | [002_print_mode_capture.md](../user_story/002_print_mode_capture.md) | Developer |
| 11 | [011_file_input.md](../user_story/011_file_input.md) | Developer |
| 12 | [012_code_block_extraction.md](../user_story/012_code_block_extraction.md) | Developer |
| 13 | [013_structured_json_pipeline.md](../user_story/013_structured_json_pipeline.md) | Developer |

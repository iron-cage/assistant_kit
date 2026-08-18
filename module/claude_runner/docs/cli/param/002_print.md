# CLI Parameter: --print

Explicit print mode flag. Print mode is already the default whenever a message is
given, stdin is not a terminal, or `--file`/piped stdin content supplies the prompt
(see [006_cli_design.md](../../feature/006_cli_design.md) § Design : Mode selection) —
this flag is a backward-compatible explicit alias for that default. Captures Claude's
stdout and prints it instead of passing through the TTY.

- **Aliases:** `-p`
- **Type:** bool (standalone flag)
- **Default:** auto (active when message given, stdin is non-TTY, or `--file`/stdin content is present; inactive for bare REPL with a real TTY and no such content)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Claude-Native Flags](../param_group/01_claude_native_flags.md)
- **JSON Key:** `"print"`

```sh
clr "Explain this function"        # print mode by default
clr -p "Explain this function"     # same — explicit alias
output=$(clr "List files" --model sonnet)
```

**Note:** Requested print mode (`-p`/`--print`, `CLR_PRINT`, or JSON config) with no
message, `--file`, or piped stdin content exits with error code 1 (see
[Design Decisions D3](../../001_design_decisions.md)).

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
| 1 | [`run`](../command/01_run.md) | auto | Default on when message given, stdin is non-TTY, or `--file`/stdin content is present |
| 5 | [`ask`](../command/05_ask.md) | auto | `ask` delegates to `run`'s dispatch — identical auto-print formula, not unconditional (see [05_ask.md](../command/05_ask.md) Execution Modes: `clr ask` with no message opens the interactive REPL) |
| 11 | [`topic`](../command/11_topic.md) | auto | Identical to `ask`; delegates to `run`'s handler |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 2 | [002_print_mode_capture.md](../user_story/002_print_mode_capture.md) | Developer |
| 11 | [011_file_input.md](../user_story/011_file_input.md) | Developer |
| 12 | [012_code_block_extraction.md](../user_story/012_code_block_extraction.md) | Developer |
| 13 | [013_structured_json_pipeline.md](../user_story/013_structured_json_pipeline.md) | Developer |

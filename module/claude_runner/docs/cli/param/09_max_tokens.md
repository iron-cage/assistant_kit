# Parameter :: 9. `--max-tokens`

Set the maximum number of output tokens for the Claude Code subprocess.
Passed via the `CLAUDE_CODE_MAX_OUTPUT_TOKENS` environment variable.

- **Type:** [`TokenLimit`](../type.md#type--3-tokenlimit)
- **Default:** 200000
- **Command:** [`run`](../command.md#command--1-run)
- **Group:** [Runner Control](../param_group.md#group--2-runner-control)
- **Validation:** must be a valid u32 (0–4294967295); non-numeric → error

```sh
clr "Summarize" --max-tokens 50000
```

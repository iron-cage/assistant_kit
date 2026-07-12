# CLI Parameter: --input-format

Select the input format for Claude Code subprocess stdin.

- **Type:** enum
- **Default:** — (Claude Code default; `text` when not specified)
- **Valid Values:** `text`, `stream-json`
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Claude-Native Flags](../param_group/01_claude_native_flags.md)
- **Validation:** requires a value; unknown value → exit 1, stderr names valid values (`text`, `stream-json`)
- **JSON Key:** `"input-format"`

```sh
clr --input-format stream-json "hi"
clr --input-format text "hi"
clr --input-format badvalue "hi"   # error: invalid --input-format value
```

### Variant Table

| Value | Origin | Forwarded | Behavior |
|-------|--------|-----------|----------|
| `text` | Claude-Native | `--input-format text` | Plain text stdin (default) |
| `stream-json` | Claude-Native | `--input-format stream-json` | Newline-delimited JSON stdin events |

### Relationship to `--output-format stream-json`

`--input-format` and `--output-format` are independent flags — either may be set
without the other. Live incremental consumption of NDJSON events on `clr`'s own
stdout is controlled solely by `--output-format stream-json` (see
[`--output-format`](061_output_format.md)); `--input-format stream-json` only
changes how `clr` labels the outbound stdin stream to the `claude` subprocess.

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | Full | `--print`, `--model`, `--verbose`, `--effort`, `--json-schema`, `--mcp-config`, `--output-format`, `--max-turns`, `--allowed-tools`, `--disallowed-tools`, `--max-budget-usd`, `--add-dir`, `--fallback-model` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | — | — |
| 5 | [`ask`](../command/05_ask.md) | — | — |

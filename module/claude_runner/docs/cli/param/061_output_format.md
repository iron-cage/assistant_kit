# CLI Parameter: --output-format

Select the output format for Claude Code subprocess response.

- **Type:** enum
- **Default:** — (Claude Code default; `text` when not specified)
- **Valid Values:** `text`, `json`, `stream-json`, `summary`
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Claude-Native Flags](../param_group/01_claude_native_flags.md)
- **Validation:** requires a value; `--output-format` at end of argv → error

```sh
clr "Explain" --output-format json
clr --output-format stream-json "Fix bug"
clr --output-format summary "Explain factorial"
```

### Variant Table

| Value | Origin | Forwarded | Behavior |
|-------|--------|-----------|----------|
| `text` | Claude-Native | `--output-format text` | Plain text output (default) |
| `json` | Claude-Native | `--output-format json` | Single JSON object on completion |
| `stream-json` | Claude-Native | `--output-format stream-json` | Newline-delimited JSON chunks |
| `summary` | Runner-synthetic | `--output-format json` | YAML metadata header + text body (see below) |

### `summary` Variant

Runner-level transformation. `clr` sends `--output-format json` to claude, parses the JSON response per [`007_json_response.md`](../../../../contract/claude_code/docs/formats/007_json_response.md), and renders:

1. **YAML header** (boxed, ANSI-colored) — all top-level attributes in YAML format; `content` array replaced with topology pseudo-attribute showing block count, per-block type, and field keys
2. **Separator** (`---`)
3. **Text body** — extracted text block content, rendered uncolored as in `text` mode

Color scheme: keys cyan, string values green, numeric values yellow, null dim, block types magenta, block indices bright-black, tool names bold-green, separator dim.

Content topology pseudo-attributes:

| Block Type | Rendering | Shown |
|------------|-----------|-------|
| `text` | `text  N chars` | Character count; content in body |
| `thinking` | `thinking  {thinking, signature}` | Field keys only |
| `tool_use` | `tool_use  "Name" {field_keys}` | Tool name + input keys |
| `tool_result` | `tool_result  ok` or `ERROR` | Success/failure indicator |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | Full | `--print`, `--model`, `--verbose`, `--effort`, `--json-schema`, `--mcp-config`, `--max-turns`, `--allowed-tools`, `--disallowed-tools`, `--max-budget-usd`, `--add-dir`, `--fallback-model` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | — | — |
| 5 | [`ask`](../command/05_ask.md) | — | — |

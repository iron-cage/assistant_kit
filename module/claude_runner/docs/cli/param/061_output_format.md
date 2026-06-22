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
| `summary` | Runner-synthetic | `--output-format json` | Key:val header + text body (see below) |

### `summary` Variant

Runner-level transformation. `clr` sends `--output-format json` to claude, parses the CLR result envelope response emitted by `claude --output-format json`, and renders:

1. **Key:val header** (ANSI-colored) — CLR result envelope fields as `key: value` lines
2. **Separator** (`---`)
3. **Text body** — the `result` field value rendered uncolored

Color scheme: keys cyan, string values green, numeric values yellow, boolean values yellow, separator dim.

CLR result envelope fields rendered in header:

| Field | Type | Header Line |
|-------|------|-------------|
| `type` | string | `type: result` |
| `subtype` | string | `subtype: success` |
| `session_id` | string | `session_id: …` |
| `is_error` | bool | `is_error: false` |
| `usage.input_tokens` | u64 | `input_tokens: N` |
| `usage.output_tokens` | u64 | `output_tokens: N` |
| `total_cost_usd` | float | `total_cost_usd: X.XXXX` |
| `result` | string | text body after `---` separator |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | Full | `--print`, `--model`, `--verbose`, `--effort`, `--json-schema`, `--mcp-config`, `--max-turns`, `--allowed-tools`, `--disallowed-tools`, `--max-budget-usd`, `--add-dir`, `--fallback-model` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | — | — |
| 5 | [`ask`](../command/05_ask.md) | — | — |

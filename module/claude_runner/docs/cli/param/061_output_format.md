# CLI Parameter: --output-format

Select the output format for Claude Code subprocess response.

- **Type:** enum
- **Default:** — (Claude Code default; `text` when not specified)
- **Valid Values:** `text`, `json`, `stream-json`, `summary`
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Claude-Native Flags](../param_group/01_claude_native_flags.md)
- **Validation:** requires a value; `--output-format` at end of argv → error
- **JSON Key:** `"output-format"`

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

### CLR Result Envelope Fields

Complete field table for the JSON object emitted by `claude --output-format json`. All fields are rendered in the key:val header by default (`--summary-fields full`). Use [`--summary-fields`](071_summary_fields.md) to select a subset.

| # | Field | JSON Path | Type | Header Line |
|---|-------|-----------|------|-------------|
| 1 | `type` | top-level | string | `type: result` |
| 2 | `subtype` | top-level | string | `subtype: success` |
| 3 | `session_id` | top-level | string | `session_id: …` |
| 4 | `uuid` | top-level | string | `uuid: …` |
| 5 | `is_error` | top-level | bool | `is_error: false` |
| 6 | `stop_reason` | top-level | string | `stop_reason: end_turn` |
| 7 | `num_turns` | top-level | u64 | `num_turns: N` |
| 8 | `fast_mode_state` | top-level | string | `fast_mode_state: off` |
| 9 | `duration_ms` | top-level | u64 | `duration_ms: N` |
| 10 | `duration_api_ms` | top-level | u64 | `duration_api_ms: N` |
| 11 | `input_tokens` | `usage` | u64 | `input_tokens: N` |
| 12 | `output_tokens` | `usage` | u64 | `output_tokens: N` |
| 13 | `cache_creation_input_tokens` | `usage` | u64 | `cache_creation_input_tokens: N` |
| 14 | `cache_read_input_tokens` | `usage` | u64 | `cache_read_input_tokens: N` |
| 15 | `total_cost_usd` | top-level | float | `total_cost_usd: X.XXXX` |
| 16 | `service_tier` | `usage` | string | `service_tier: standard` |
| 17 | `speed` | `usage` | string | `speed: standard` |
| 18 | `inference_geo` | `usage` | string | `inference_geo: …` (dim if empty) |
| 19 | `web_search_requests` | `usage.server_tool_use` | u64 | `web_search_requests: N` |
| 20 | `web_fetch_requests` | `usage.server_tool_use` | u64 | `web_fetch_requests: N` |
| 21 | `cache_ephemeral_1h_input_tokens` | `usage.cache_creation` | u64 | `cache_ephemeral_1h_input_tokens: N` |
| 22 | `cache_ephemeral_5m_input_tokens` | `usage.cache_creation` | u64 | `cache_ephemeral_5m_input_tokens: N` |
| 23 | `model` | `modelUsage` key | string | `model: claude-opus-4-6` |
| 24 | `model_input_tokens` | `modelUsage.<m>` | u64 | `model_input_tokens: N` |
| 25 | `model_output_tokens` | `modelUsage.<m>` | u64 | `model_output_tokens: N` |
| 26 | `model_cache_read_input_tokens` | `modelUsage.<m>` | u64 | `model_cache_read_input_tokens: N` |
| 27 | `model_cache_creation_input_tokens` | `modelUsage.<m>` | u64 | `model_cache_creation_input_tokens: N` |
| 28 | `model_web_search_requests` | `modelUsage.<m>` | u64 | `model_web_search_requests: N` |
| 29 | `model_cost_usd` | `modelUsage.<m>` | float | `model_cost_usd: X.XXXX` |
| 30 | `model_context_window` | `modelUsage.<m>` | u64 | `model_context_window: N` |
| 31 | `model_max_output_tokens` | `modelUsage.<m>` | u64 | `model_max_output_tokens: N` |
| 32 | `permission_denials` | top-level | count | `permission_denials: N` |
| — | `result` | top-level | string | text body after `---` separator (always rendered, not filterable) |

**`modelUsage` note:** The `modelUsage` object is keyed by model name (e.g., `"claude-opus-4-6"`). When multiple models are used (fallback), the first model's stats are rendered. Fields 23–31 flatten the nested per-model object into `model_*` prefixed header lines.

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | Full | `--print`, `--model`, `--verbose`, `--effort`, `--json-schema`, `--mcp-config`, `--max-turns`, `--allowed-tools`, `--disallowed-tools`, `--max-budget-usd`, `--add-dir`, `--fallback-model` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | — | — |
| 5 | [`ask`](../command/05_ask.md) | — | — |

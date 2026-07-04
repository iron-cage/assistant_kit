# CLI Parameter: --summary-fields

Select which CLR result envelope fields `render_summary()` includes in the key:val header.
Accepts preset profile names or a custom comma-separated field whitelist.

- **Type:** string (preset name or comma-separated field list)
- **Default:** `full`
- **Valid Values:** `minimal`, `standard`, `full`, or comma-separated field names
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **Requires:** `--output-style summary` (the default); ignored when `--output-style raw`
- **JSON Key:** `"summary-fields"`

```sh
clr -p "task"                                       # default: full (all 32 header fields)
clr -p "task" --summary-fields full                 # explicit full
clr -p "task" --summary-fields standard             # 14 key fields
clr -p "task" --summary-fields minimal              # 7 fields (v1.2.0 compat)
clr -p "task" --summary-fields type,session_id,total_cost_usd  # custom whitelist
CLR_SUMMARY_FIELDS=minimal clr -p "task"           # env-var equivalent
clr -p "task" --summary-fields bogus               # exit 1: unknown profile/field
```

### Preset Profiles

| Profile | Fields | Count | Purpose |
|---------|--------|------:|---------|
| `full` | all renderable fields | 32 | Complete envelope visibility (default) |
| `standard` | type, subtype, session_id, is_error, stop_reason, num_turns, duration_ms, input_tokens, output_tokens, cache_creation_input_tokens, cache_read_input_tokens, total_cost_usd, service_tier, model | 14 | Key operational fields |
| `minimal` | type, subtype, session_id, is_error, input_tokens, output_tokens, total_cost_usd | 7 | Backward compatible with v1.2.0 rendering |

### Custom Whitelist

A comma-separated list of field names (no spaces). Only listed fields are rendered; order follows the canonical field order in `render_summary()`, not the order specified in the value. Unknown field names cause exit 1.

```sh
clr -p "task" --summary-fields "session_id,total_cost_usd,duration_ms"
# renders:
#   session_id: 03b6adee-…
#   duration_ms: 2441
#   total_cost_usd: 0.2226
#   ---
#   hello
```

The `result` field (text body after `---`) is always rendered regardless of field selection. Only header fields are filterable.

### Valid Field Names

All field names correspond to the flattened CLR result envelope keys rendered by `render_summary()`. See [`061_output_format.md`](061_output_format.md) § CLR Result Envelope Fields for the complete field table with JSON paths and types.

### Interaction with `--output-style`

| `--output-style` | `--summary-fields` | Behavior |
|-------------------|---------------------|----------|
| `summary` (default) | any value | Fields filtered per profile/whitelist |
| `raw` | any value | `--summary-fields` ignored; raw output passthrough |

### Validation

- Unknown profile name AND not a valid comma-separated field list: exit 1 with `invalid summary-fields '{v}' — unknown profile or field name`
- Custom list containing unknown field: exit 1 with `invalid summary-fields: unknown field '{f}' — see --help for valid field names`

### Env var

`CLR_SUMMARY_FIELDS` — applied when `--summary-fields` is absent; accepts profile names and custom whitelists; invalid values exit 1 with `CLR_SUMMARY_FIELDS: invalid value '{v}' — unknown profile or field name`. CLI flag wins when both set (standard precedence).

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | `--output-style`, `--dry-run`, `--trace`, ... |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | `full` | Applied in `run_print_mode()` via field filter in `render_summary()` |
| 5 | [`ask`](../command/05_ask.md) | `full` | Pure alias for run; same rendering path |

### See Also

- [`070_output_style.md`](070_output_style.md) — controls whether `render_summary()` fires at all
- [`061_output_format.md`](061_output_format.md) — CLR result envelope schema and field table
- [`summary.rs`](../../../src/cli/summary.rs) — `render_summary()` implementation

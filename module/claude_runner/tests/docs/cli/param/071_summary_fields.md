# Parameter :: `--summary-fields` (run/ask)

Edge case coverage for the `--summary-fields` parameter on the `run`/`ask` dispatch paths. See [071_summary_fields.md](../../../../docs/cli/param/071_summary_fields.md) for specification.

**Scope note:** `--summary-fields` controls which CLR result envelope fields appear in the key:val header rendered by `render_summary()`. Only effective when `--output-style summary` (default). The `result` text body after `---` is always rendered regardless of field selection.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-01 | Default (`full`) — stdout contains all 32 header fields | Behavioral Divergence |
| EC-02 | `--summary-fields full` explicit — same as default | Explicit |
| EC-03 | `--summary-fields minimal` — stdout contains 7 fields only | Behavioral Divergence |
| EC-04 | `--summary-fields standard` — stdout contains 14 fields | Profile |
| EC-05 | Custom whitelist `type,session_id,total_cost_usd` — only 3 fields | Custom |
| EC-06 | `--output-style raw` ignores `--summary-fields` — no header | Interaction |
| EC-07 | Invalid profile name → exit 1; stderr contains validation msg | Validation |
| EC-08 | Custom with unknown field → exit 1; stderr names the field | Validation |
| EC-09 | `CLR_SUMMARY_FIELDS=minimal` env var — 7 fields | Env Var |
| EC-10 | `CLR_SUMMARY_FIELDS=minimal` + `--summary-fields full` → flag wins | CLI-wins |
| EC-11 | `CLR_SUMMARY_FIELDS=bogus` → exit 1; stderr contains validation msg | Env Var Validation |
| EC-12 | `--summary-fields minimal` — `result` text body still rendered after `---` | Result Preserved |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-01, EC-03)
- Explicit: 1 test (EC-02)
- Profile: 1 test (EC-04)
- Custom: 1 test (EC-05)
- Interaction: 1 test (EC-06)
- Validation: 2 tests (EC-07, EC-08)
- Env Var: 1 test (EC-09)
- CLI-wins: 1 test (EC-10)
- Env Var Validation: 1 test (EC-11)
- Result Preserved: 1 test (EC-12)

**Total:** 12 test cases

## Architectural Constraint

All 12 tests use a fake `claude` subprocess emitting the full CLR result envelope JSON:

```json
{"type":"result","subtype":"success","session_id":"00000000-0000-0000-0000-000000000001","is_error":false,"duration_ms":100,"duration_api_ms":90,"num_turns":1,"result":"hello","stop_reason":"end_turn","total_cost_usd":0.001,"uuid":"00000000-0000-0000-0000-000000000002","fast_mode_state":"off","usage":{"input_tokens":3,"output_tokens":4,"cache_creation_input_tokens":0,"cache_read_input_tokens":0,"service_tier":"standard","speed":"standard","inference_geo":"","server_tool_use":{"web_search_requests":0,"web_fetch_requests":0},"cache_creation":{"ephemeral_1h_input_tokens":0,"ephemeral_5m_input_tokens":0},"iterations":[]},"modelUsage":{"claude-opus-4-6":{"inputTokens":3,"outputTokens":4,"cacheReadInputTokens":0,"cacheCreationInputTokens":0,"webSearchRequests":0,"costUSD":0.001,"contextWindow":200000,"maxOutputTokens":32000}},"permission_denials":[]}
```

Field presence is asserted by checking `stdout.contains("field_name:")` for each expected field and `!stdout.contains("field_name:")` for excluded fields.

---

### EC-01: Default `full` — all 32 header fields present

- **Given:** no `--summary-fields` flag; no `CLR_SUMMARY_FIELDS`; fake claude fixture; `-p --max-sessions 0`
- **When:** `clr -p --max-sessions 0 "x"` with fake claude emitting full envelope JSON
- **Then:** Exit 0; stdout contains `---`; stdout contains `duration_ms:`, `uuid:`, `model:`, `permission_denials:` (fields absent in v1.2.0)
- **Exit:** 0
- **Source:** [071_summary_fields.md](../../../../docs/cli/param/071_summary_fields.md) Default (`full`)

---

### EC-02: Explicit `--summary-fields full` — same as default

- **Given:** `--summary-fields full`; fake claude fixture; `-p --max-sessions 0`
- **When:** `clr -p --max-sessions 0 --summary-fields full "x"` with fake claude
- **Then:** Exit 0; stdout contains `duration_ms:` and `model:`; identical to EC-01
- **Exit:** 0
- **Source:** [071_summary_fields.md](../../../../docs/cli/param/071_summary_fields.md) Preset Profiles (`full`)

---

### EC-03: `--summary-fields minimal` — 7 fields only

- **Given:** `--summary-fields minimal`; fake claude fixture; `-p --max-sessions 0`
- **When:** `clr -p --max-sessions 0 --summary-fields minimal "x"` with fake claude
- **Then:** Exit 0; stdout contains `type:`, `subtype:`, `session_id:`, `is_error:`, `input_tokens:`, `output_tokens:`, `total_cost_usd:`; stdout does NOT contain `duration_ms:`, `uuid:`, `model:`, `stop_reason:`
- **Exit:** 0
- **Source:** [071_summary_fields.md](../../../../docs/cli/param/071_summary_fields.md) Preset Profiles (`minimal`)

---

### EC-04: `--summary-fields standard` — 14 fields

- **Given:** `--summary-fields standard`; fake claude fixture; `-p --max-sessions 0`
- **When:** `clr -p --max-sessions 0 --summary-fields standard "x"` with fake claude
- **Then:** Exit 0; stdout contains `stop_reason:`, `num_turns:`, `duration_ms:`, `cache_creation_input_tokens:`, `service_tier:`, `model:`; stdout does NOT contain `uuid:`, `fast_mode_state:`, `duration_api_ms:`, `model_context_window:`
- **Exit:** 0
- **Source:** [071_summary_fields.md](../../../../docs/cli/param/071_summary_fields.md) Preset Profiles (`standard`)

---

### EC-05: Custom whitelist — only specified fields

- **Given:** `--summary-fields type,session_id,total_cost_usd`; fake claude fixture; `-p --max-sessions 0`
- **When:** `clr -p --max-sessions 0 --summary-fields "type,session_id,total_cost_usd" "x"` with fake claude
- **Then:** Exit 0; stdout contains `type:`, `session_id:`, `total_cost_usd:`; stdout does NOT contain `subtype:`, `is_error:`, `input_tokens:`, `duration_ms:`
- **Exit:** 0
- **Source:** [071_summary_fields.md](../../../../docs/cli/param/071_summary_fields.md) Custom Whitelist

---

### EC-06: `--output-style raw` ignores `--summary-fields` — no header

- **Given:** `--output-style raw`; `--summary-fields minimal`; fake claude fixture; `-p --max-sessions 0`
- **When:** `clr -p --max-sessions 0 --output-style raw --summary-fields minimal "x"` with fake claude
- **Then:** Exit 0; stdout does NOT contain `---`; `render_summary()` not called; `--summary-fields` silently ignored
- **Exit:** 0
- **Source:** [071_summary_fields.md](../../../../docs/cli/param/071_summary_fields.md) Interaction with `--output-style`

---

### EC-07: Invalid profile name → exit 1

- **Given:** `--summary-fields bogus`
- **When:** `clr --summary-fields bogus`
- **Then:** Exit 1; stderr contains `"invalid summary-fields 'bogus'"`
- **Exit:** 1
- **Source:** [071_summary_fields.md](../../../../docs/cli/param/071_summary_fields.md) Validation

---

### EC-08: Custom whitelist with unknown field → exit 1

- **Given:** `--summary-fields type,nonexistent_field`
- **When:** `clr --summary-fields "type,nonexistent_field"`
- **Then:** Exit 1; stderr contains `"unknown field 'nonexistent_field'"`
- **Exit:** 1
- **Source:** [071_summary_fields.md](../../../../docs/cli/param/071_summary_fields.md) Validation

---

### EC-09: `CLR_SUMMARY_FIELDS=minimal` → 7 fields

- **Given:** `CLR_SUMMARY_FIELDS=minimal`; no `--summary-fields` flag; fake claude fixture; `-p --max-sessions 0`
- **When:** `CLR_SUMMARY_FIELDS=minimal clr -p --max-sessions 0 "x"` with fake claude
- **Then:** Exit 0; stdout does NOT contain `duration_ms:`; stdout contains `type:`, `total_cost_usd:`
- **Exit:** 0
- **Source:** [071_summary_fields.md](../../../../docs/cli/param/071_summary_fields.md) Env var

---

### EC-10: `CLR_SUMMARY_FIELDS=minimal` + `--summary-fields full` → flag wins

- **Given:** `CLR_SUMMARY_FIELDS=minimal`; `--summary-fields full`; fake claude fixture; `-p --max-sessions 0`
- **When:** `CLR_SUMMARY_FIELDS=minimal clr -p --max-sessions 0 --summary-fields full "x"` with fake claude
- **Then:** Exit 0; stdout contains `duration_ms:` and `model:`; CLI flag wins over env var
- **Exit:** 0
- **Source:** [071_summary_fields.md](../../../../docs/cli/param/071_summary_fields.md) Env var (CLI flag wins)

---

### EC-11: `CLR_SUMMARY_FIELDS=bogus` → exit 1

- **Given:** `CLR_SUMMARY_FIELDS=bogus`; no `--summary-fields` flag
- **When:** `CLR_SUMMARY_FIELDS=bogus clr run -m "x"`
- **Then:** Exit 1; stderr contains `"CLR_SUMMARY_FIELDS: invalid value 'bogus'"`
- **Exit:** 1
- **Source:** [071_summary_fields.md](../../../../docs/cli/param/071_summary_fields.md) Env var validation

---

### EC-12: `--summary-fields minimal` — result text body always rendered

- **Given:** `--summary-fields minimal`; fake claude fixture with `"result":"hello world"`; `-p --max-sessions 0`
- **When:** `clr -p --max-sessions 0 --summary-fields minimal "x"` with fake claude
- **Then:** Exit 0; stdout contains `---` separator; stdout contains `hello world`; result body is not filtered by `--summary-fields`
- **Exit:** 0
- **Source:** [071_summary_fields.md](../../../../docs/cli/param/071_summary_fields.md) Custom Whitelist ("result field value … always rendered")

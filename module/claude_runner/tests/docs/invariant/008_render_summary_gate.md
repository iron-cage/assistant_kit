# Test: Invariant — render_summary() Gate Field

Test case planning for [invariant/008_render_summary_gate.md](../../../../docs/invariant/008_render_summary_gate.md). Tests validate that `render_summary()` returns `Some(_)` for any CLR result envelope containing `"type":"result"`, regardless of which optional fields are absent, and returns `None` only for non-CLR-result input.

**Source:** [invariant/008_render_summary_gate.md](../../../../docs/invariant/008_render_summary_gate.md)
**Related:** [cli/param/070_output_style.md](../cli/param/070_output_style.md), [docs/001_design_decisions.md](../../../../docs/001_design_decisions.md) (D15)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Minimal 7-field CLR envelope (no `session_id`) → `render_summary()` returns `Some(_)` | Invariant Hold |
| IT-2 | Minimal 7-field CLR envelope → integration: `clr -p` stdout contains `---` | Invariant Hold |
| IT-3 | Full CLR envelope with `session_id` → `render_summary()` returns `Some(_)` (regression guard) | Regression Guard |
| IT-4 | JSON with `"type":"message"` → `render_summary()` returns `None` | Invariant Boundary |
| IT-5 | JSON without `type` field → `render_summary()` returns `None` | Invariant Boundary |
| IT-6 | Non-JSON input → `render_summary()` returns `None` | Invariant Boundary |
| IT-7 | Source does NOT contain `extract_str( json, "session_id" )?` — anti-pattern absent | Structural |

## Test Coverage Summary

- Invariant Hold: 2 tests (IT-1, IT-2)
- Regression Guard: 1 test (IT-3)
- Invariant Boundary: 3 tests (IT-4, IT-5, IT-6)
- Structural: 1 test (IT-7)

**Total:** 7 invariant test cases

## Architectural Constraint

IT-1, IT-3, IT-4, IT-5, IT-6 are unit tests in `tests/summary_unit_test.rs` that call `render_summary()` directly with crafted JSON strings — no subprocess needed.

IT-2 is an integration test in `tests/output_style_test.rs` (EC-14) using a fake `claude` subprocess that emits a 7-field minimal CLR envelope; the full `clr -p` execution path verifies the rendering end-to-end.

IT-7 is a structural test (source code search) in `tests/output_style_test.rs` that asserts `src/cli/summary.rs` does NOT contain the anti-pattern string `extract_str( json, "session_id" )?` (with `?`). This prevents BUG-310 from being silently re-introduced in future refactors.

## Implementation Notes

| IT | Test Function | File |
|----|---------------|------|
| IT-1 | `render_summary_accepts_envelope_without_session_id` | `tests/summary_unit_test.rs` |
| IT-2 | `ec14_render_summary_minimal_envelope_no_session_id` | `tests/output_style_test.rs` |
| IT-3 | `ec14_render_summary_clr_envelope_accepted` | `tests/summary_unit_test.rs` |
| IT-4 | `render_summary_rejects_non_result_type` | `tests/summary_unit_test.rs` |
| IT-5 | `render_summary_rejects_json_without_type` | `tests/summary_unit_test.rs` |
| IT-6 | `render_summary_rejects_non_json` | `tests/summary_unit_test.rs` |
| IT-7 | `render_summary_gate_uses_type_not_session_id` | `tests/output_style_test.rs` |

---

### IT-1: Minimal 7-field CLR envelope → `render_summary()` returns `Some(_)`

- **Given:** JSON string `{"type":"result","subtype":"success","is_error":false,"duration_ms":1000,"duration_api_ms":900,"num_turns":1,"result":"hello"}` (no `session_id`, no `usage`, no `total_cost_usd`)
- **When:** `render_summary(json, None)` called directly (unit test)
- **Then:** Returns `Some(rendered)` — gate `msg_type == "result"` is satisfied; `session_id.unwrap_or_default()` handles absence; rendered output contains `---` separator and `hello` result text
- **Exit:** N/A (unit test; assertion: `assert!(result.is_some())`)
- **Source:** [invariant/008_render_summary_gate.md](../../../../docs/invariant/008_render_summary_gate.md) Invariant Statement; BUG-310 regression coverage

---

### IT-2: Minimal CLR envelope → integration: `clr -p` stdout contains `---`

- **Given:** fake claude subprocess emitting `{"type":"result","subtype":"success","is_error":false,"duration_ms":1000,"duration_api_ms":900,"num_turns":1,"result":"hello"}`; `-p --max-sessions 0`; no `--output-style` (default `summary`); no `--output-format` (auto-inject fires)
- **When:** `clr -p --max-sessions 0 "x"` with minimal-envelope fake claude
- **Then:** Exit 0; stdout contains `---`; render_summary returned `Some(_)` through the full execution path
- **Exit:** 0
- **Source:** [invariant/008_render_summary_gate.md](../../../../docs/invariant/008_render_summary_gate.md) Invariant Statement; EC-14 in [070_output_style.md test spec](../cli/param/070_output_style.md)

---

### IT-3: Full CLR envelope → `render_summary()` returns `Some(_)` (regression guard)

- **Given:** JSON string with all fields including `session_id`, `usage`, `total_cost_usd`
- **When:** `render_summary(json, None)` called directly (unit test)
- **Then:** Returns `Some(rendered)` — gate satisfied; all field extractors return real values; rendered output contains all expected fields
- **Exit:** N/A (unit test; assertion: `assert!(result.is_some())`)
- **Source:** [invariant/008_render_summary_gate.md](../../../../docs/invariant/008_render_summary_gate.md) Invariant Statement; regression guard for EC-01..EC-13 parity

---

### IT-4: JSON with `"type":"message"` → `render_summary()` returns `None`

- **Given:** JSON string `{"type":"message","content":"some stream message"}`
- **When:** `render_summary(json, None)` called directly (unit test)
- **Then:** Returns `None` — `msg_type != "result"` gate fires; non-CLR-result input is rejected
- **Exit:** N/A (unit test; assertion: `assert!(result.is_none())`)
- **Source:** [invariant/008_render_summary_gate.md](../../../../docs/invariant/008_render_summary_gate.md) Invariant Statement (condition 2)

---

### IT-5: JSON without `type` field → `render_summary()` returns `None`

- **Given:** JSON string `{"session_id":"abc","result":"hello","is_error":false}` (valid JSON, no `type` key)
- **When:** `render_summary(json, None)` called directly (unit test)
- **Then:** Returns `None` — `extract_str(json, "type")?` returns `None` (field absent); gate propagates `None` via `?`; non-CLR envelope rejected
- **Exit:** N/A (unit test; assertion: `assert!(result.is_none())`)
- **Source:** [invariant/008_render_summary_gate.md](../../../../docs/invariant/008_render_summary_gate.md) Invariant Statement (condition 2)

---

### IT-6: Non-JSON input → `render_summary()` returns `None`

- **Given:** Plain text string `"this is not json at all"`
- **When:** `render_summary(text, None)` called directly (unit test)
- **Then:** Returns `None` — all `extract_str()` calls return `None` for non-JSON; gate propagates `None`; raw text bypassed
- **Exit:** N/A (unit test; assertion: `assert!(result.is_none())`)
- **Source:** [invariant/008_render_summary_gate.md](../../../../docs/invariant/008_render_summary_gate.md) Invariant Statement (condition 1)

---

### IT-7: Source does NOT contain `extract_str( json, "session_id" )?` (structural)

- **Given:** Source file `src/cli/summary.rs`
- **When:** File contents read at test run time
- **Then:** File does NOT contain the substring `extract_str( json, "session_id" )?` — the BUG-310 anti-pattern is absent; gate is on `type` field, not `session_id`
- **Exit:** 0
- **Source:** [invariant/008_render_summary_gate.md](../../../../docs/invariant/008_render_summary_gate.md) Enforcement Mechanism; Anti-pattern section

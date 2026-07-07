# Test: Invariant — JSON String Extraction Escape Handling

Test case planning for [invariant/014_json_string_extraction_escape_handling.md](../../../docs/invariant/014_json_string_extraction_escape_handling.md). Tests validate that every hand-rolled JSON string-value terminator scan in the crate (`try_jsonl_task()`, `parse_json_str()`, `render_summary()`'s inline `model_name` extraction, `extract_str()`) correctly bounds a value in the presence of an escaped quote (requirement 1) and returns `None` rather than a truncated partial value when the scan exhausts without finding an unescaped closing quote (requirement 2).

**Source:** [invariant/014_json_string_extraction_escape_handling.md](../../../docs/invariant/014_json_string_extraction_escape_handling.md)
**Related:** [invariant/008_render_summary_gate.md](../../../docs/invariant/008_render_summary_gate.md) (`render_summary()`'s overall gate), [invariant/009_session_mismatch_detection.md](../../../docs/invariant/009_session_mismatch_detection.md) (`extract_session_id()` consumer affected by a requirement-2 violation)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | `try_jsonl_task()`: JSONL content with an escaped `"` in the human message → extracted task text is not truncated at the escaped quote | Invariant Hold |
| IN-2 | `parse_json_str()`: gate-state `cwd` field containing an escaped `"` (BUG-384 write-side output) → extracted `cwd` is not truncated at the escaped quote | Invariant Hold |
| IN-3 | `render_summary()` inline `model_name`: `modelUsage` object key containing an escaped `"` → extracted model name is not truncated at the escaped quote | Invariant Hold |
| IN-4 | `extract_str()`: JSON envelope with an escaped `"` inside the target field's value → extracted value is not truncated at the escaped quote (regression guard — this site was already correct pre-BUG-394; must remain so) | Regression Guard |
| IN-5 | `extract_str()`: JSON envelope where the target field's string value has no closing quote before input ends → returns `None`, not `Some(<truncated>)` | Invariant Boundary |
| IN-6 | `extract_session_id()`: `session_id` field value truncated (unterminated) → returns `None`; BUG-320 mismatch warning does NOT fire | Invariant Boundary |
| IN-7 | `render_summary()`'s `"result"` field: unterminated value → falls back to empty string via `.unwrap_or_default()`, not a truncated partial string | Invariant Boundary |

## Test Coverage Summary

- Invariant Hold: 3 tests (IN-1, IN-2, IN-3) — requirement 1, the 3 BUG-394 sites
- Regression Guard: 1 test (IN-4) — requirement 1, the already-correct baseline site
- Invariant Boundary: 3 tests (IN-5, IN-6, IN-7) — requirement 2, BUG-395's site and its two downstream consumers

**Total:** 7 invariant test cases

## Architectural Constraint

IN-1 is an integration test in `tests/ps_command_test.rs`, extending the existing `it_16_task_column_form_a` harness (`tests/ps_command_test.rs:506-555`) with an escaped-quote content fixture — it already builds the exact HOME/CLR_PROC_DIR/JSONL fixture this case needs; only the JSONL `content` value changes.

IN-2 is an integration test in `tests/ps_command_test.rs` (as implemented — see the Implementation Notes note below for why this differs from the file originally planned here), verifying `parse_json_str()` correctly reverses the BUG-384 write-side escaping when `clr ps` renders the "Queued CLR Processes" table.

IN-3, IN-4, IN-6 are unit tests in `tests/summary_unit_test.rs` that call the relevant `summary.rs` function directly (or through its nearest public wrapper) with crafted JSON strings — no subprocess needed. IN-5 has no standalone test function — see the Implementation Notes note on its disposition.

IN-7 is a unit test in `tests/summary_unit_test.rs` calling `render_summary()` directly with a crafted envelope whose `"result"` value is unterminated.

## Implementation Notes

| ID | Test Function | File | Status |
|----|---------------|------|--------|
| IN-1 | `it_36_task_column_escaped_quote_not_truncated` | `tests/ps_command_test.rs` | ✅ |
| IN-2 | `it_37_queued_table_cwd_escaped_quote_not_truncated` | `tests/ps_command_test.rs` | ✅ |
| IN-3 | `render_summary_model_name_escaped_quote_not_truncated` | `tests/summary_unit_test.rs` | ✅ |
| IN-4 | `extract_str_result_field_escaped_quote_not_truncated` | `tests/summary_unit_test.rs` | ✅ |
| IN-5 | *(covered transitively — see note)* | `tests/summary_unit_test.rs` | ✅ |
| IN-6 | `extract_session_id_returns_none_for_unterminated_session_id` | `tests/summary_unit_test.rs` | ✅ |
| IN-7 | `render_summary_result_field_unterminated_falls_back_to_empty` | `tests/summary_unit_test.rs` | ✅ |

**Note on implementation status:** all 7 cases are now `✅` — BUG-394 and BUG-395 were both fixed and manually verified on 2026-07-07 (1154/1154 crate tests passed, `./verb/lint` clean; see both bugs' `## Verification Record` in `bug/closed/`).

**Note on IN-2's actual file:** implemented in `tests/ps_command_test.rs` (as `it_37_...`), not `tests/concurrency_gate_test.rs` as originally planned in the Architectural Constraint below. `it_10_gate_file_present_shows_queued_table` in `ps_command_test.rs` already establishes a simpler, non-flaky pattern for exercising `build_queued_table()`/`parse_json_str()` — writing a synthetic `{live_pid}.json` gate-state file directly (using the test process's own PID so the `/proc/{pid}` liveness filter passes trivially) and invoking `clr ps` via `run_cli_with_env()` — rather than spawning and racing real blocked/occupier `clr` subprocesses as T07/T13 do. Both approaches exercise the identical `parse_json_str()` read path; the simpler one was chosen to avoid the 500ms-sleep timing dependency T07/T13 carry for no added coverage value here.

**Note on IN-5's disposition:** `extract_str()` is a private (non-`pub`, non-`pub(super)`) function in `summary.rs`, not exported from the crate root — it cannot be "called directly" from an external `tests/` integration-test crate as originally specified. Its only two externally-reachable call paths are `extract_session_id()` (`?`-propagation — now covered by IN-6) and `render_summary()`'s per-field `.unwrap_or_default()` extractions (covered for the `"result"` field by IN-4's escaped-quote case and IN-7's unterminated case). A literal third `extract_str`-direct test is not possible without adding test-only public surface to a function that already has complete behavioral coverage through its two real callers — IN-6 and IN-7 jointly exercise both of `extract_str`'s distinct failure-propagation shapes (`?` vs `.unwrap_or_default()`), which is what IN-5 was intended to establish independent of any one consumer.

---

### IN-1: `try_jsonl_task()` — escaped `"` in human message does not truncate task text

- **Given:** a JSONL session file whose most recent Form-A user line is `{"type":"user","content":"He said \"hi\" and left"}` (as implemented — shorter than originally drafted here to keep the correctly-bounded text under the 35-char truncation limit and avoid a partial-truncation edge case unrelated to what this case tests)
- **When:** `clr ps` is invoked and renders the active-sessions table's Task column for that session's process
- **Then:** the Task column preview reflects the content bounded correctly at the true closing quote — `He said \"hi\" and left` (note: `try_jsonl_task()` bounds but does not unescape, so the literal backslashes remain in the displayed text; this is documented as sufficient for this call site's display use) — not `He said \` truncated at the escaped quote 9 bytes in
- **Note:** `bug_reproducer(BUG-394)` — reproduces BUG-394 site 1 (`ps.rs:850`); implemented as `it_36_task_column_escaped_quote_not_truncated` in `tests/ps_command_test.rs`
- **Source:** [invariant/014_json_string_extraction_escape_handling.md](../../../docs/invariant/014_json_string_extraction_escape_handling.md) Invariant Statement requirement 1, `try_jsonl_task()` row

---

### IN-2: `parse_json_str()` — escaped `"` in gate-state `cwd` does not truncate CWD display

- **Given:** a gate-state file directly written (as implemented — no live blocked `clr` subprocess needed, see the Implementation Notes note above) with content `{"cwd":"/tmp/proj-\"quoted\"-dir","since":1720000000,"attempt":2,"message":"waiting for session slot"}`, at `$CLR_GATE_DIR/{live_pid}.json` using the test process's own PID so the `/proc/{pid}` liveness filter passes
- **When:** `clr ps` is invoked (via `run_cli_with_env`) with `CLR_GATE_DIR` pointing at that file, rendering the "Queued CLR Processes" table
- **Then:** the CWD column shows the full path bounded correctly at the true closing quote — `/tmp/proj-\"quoted\"-dir` (note: `parse_json_str()` bounds but does not unescape, so the literal backslashes remain in the displayed text) — not `/tmp/proj-\` truncated at the escaped quote
- **Note:** `bug_reproducer(BUG-394)` — reproduces BUG-394 site 2 (`ps.rs:863`); this is the unpaired read side of the BUG-384 write-side round-trip; implemented as `it_37_queued_table_cwd_escaped_quote_not_truncated` in `tests/ps_command_test.rs`, closing the exact gap BUG-384's own T07/T13 left per `## Why Not Caught` (those tests read the gate-state file directly and never invoke `clr ps`)
- **Source:** [invariant/014_json_string_extraction_escape_handling.md](../../../docs/invariant/014_json_string_extraction_escape_handling.md) Invariant Statement requirement 1, `parse_json_str()` row

---

### IN-3: `render_summary()` inline `model_name` — escaped `"` in `modelUsage` key does not truncate model name

- **Given:** a CLR result envelope whose `modelUsage` object's first key is an escaped-quote-containing model identifier, e.g. `{"modelUsage":{"He said \"hi\"-model":{"inputTokens":10, ...}}, ...}`
- **When:** `render_summary(json, None)` is called directly with the `model` field selected
- **Then:** the rendered `model:` line shows the full key text bounded at the true closing quote, not truncated at the escaped quote
- **Note:** `bug_reproducer(BUG-394)` — reproduces BUG-394 site 3 (`summary.rs:330,332`); lower practical likelihood than IN-1/IN-2 since model identifiers are internally-controlled by `claude`, included for completeness per the invariant's full-coverage requirement; implemented as `render_summary_model_name_escaped_quote_not_truncated` in `tests/summary_unit_test.rs`
- **Source:** [invariant/014_json_string_extraction_escape_handling.md](../../../docs/invariant/014_json_string_extraction_escape_handling.md) Invariant Statement requirement 1, `model_name` row

---

### IN-4: `extract_str()` — escaped `"` in target field value does not truncate (regression guard)

- **Given:** JSON string `{"type":"result","subtype":"success","is_error":false,"result":"He said \"hi\" to me","usage":{"input_tokens":0,"output_tokens":0},"total_cost_usd":0.0}`
- **When:** `render_summary(json, None)` called (`extract_str()` itself is a private, non-exported function — not directly callable from an external `tests/` crate; `render_summary()`'s `"result"` field is its nearest public wrapper, see the Implementation Notes note above)
- **Then:** the rendered output contains `He said "hi" to me` (fully unescaped form — `extract_str()`, unlike `find_unescaped_quote()`, unescapes its captured contents) — the escape-tracking loop already present in `extract_str()` correctly bounds the value at the true closing quote; this behavior must NOT regress when the requirement-2 fix (this same function's `None`-on-exhaustion fallback) is applied
- **Note:** regression guard — `extract_str()` was already escape-aware before BUG-394/BUG-395; this case pins that correctness so the requirement-2 fix (adding the `None` fallback) does not accidentally disturb the existing escape-tracking loop body; implemented as `extract_str_result_field_escaped_quote_not_truncated` in `tests/summary_unit_test.rs`
- **Source:** [invariant/014_json_string_extraction_escape_handling.md](../../../docs/invariant/014_json_string_extraction_escape_handling.md) Invariant Statement table, `extract_str()` row ("✅ already correct" for requirement 1)

---

### IN-5: `extract_str()` — unterminated string value returns `None`, not `Some(<truncated>)`

- **Given:** JSON string `{"type":"result","result":"partial output that never closes` (no closing quote before input ends)
- **When:** *(no standalone test — see the Implementation Notes note on IN-5's disposition above)* `extract_str()` is private and not directly callable from an external `tests/` crate; this exact scenario is exercised through its two real callers instead — IN-6 (`extract_session_id`, the `?`-propagation shape) and IN-7 (`render_summary`'s `"result"` field, the `.unwrap_or_default()` shape)
- **Then:** returns `None` — not `Some("partial output that never closes")` — as verified transitively by IN-6 and IN-7
- **Note:** `bug_reproducer(BUG-395)` — reproduces BUG-395's root defect directly: the loop exhausts `inner.chars()` without finding an unescaped `"`, and must fall through to an explicit `None` rather than the pre-fix trailing `Some(out)`
- **Source:** [invariant/014_json_string_extraction_escape_handling.md](../../../docs/invariant/014_json_string_extraction_escape_handling.md) Invariant Statement requirement 2

---

### IN-6: `extract_session_id()` — truncated `session_id` value returns `None`; no false-positive BUG-320 warning

- **Given:** JSON string `{"type":"result","session_id":"abc-123-truncated-no-closing-quote` (unterminated `session_id` value)
- **When:** `extract_session_id(json)` called directly (unit test)
- **Then:** returns `None` — `extract_str(json, "session_id")`'s `?`-propagation now correctly returns `None` per IN-5's fix, rather than `Some(<truncated-uuid>)`
- **Note:** downstream consequence of IN-5 — without this fix, `execution.rs`'s `if let Some(actual) = extract_session_id(...)` block would execute with a truncated partial UUID that essentially never coincidentally equals the expected UUID, firing a false-positive `"[Runner] warning: session mismatch ... (BUG-320 detected)"` that misdiagnoses a truncated envelope as session drift; post-fix, the `if let Some(...)` guard simply does not execute — no verdict is asserted either way, the honest outcome when the actual session id genuinely could not be recovered; implemented as `extract_session_id_returns_none_for_unterminated_session_id` in `tests/summary_unit_test.rs`
- **Source:** [invariant/014_json_string_extraction_escape_handling.md](../../../docs/invariant/014_json_string_extraction_escape_handling.md) § Violation Consequences, requirement 2; cross-ref [invariant/009_session_mismatch_detection.md](../../../docs/invariant/009_session_mismatch_detection.md)

---

### IN-7: `render_summary()`'s `"result"` field — unterminated value falls back to empty string, not partial text

- **Given:** JSON string `{"type":"result","subtype":"success","is_error":false,"duration_ms":1,"duration_api_ms":1,"num_turns":1,"result":"partial text that never closes` (unterminated `result` value; envelope otherwise well-formed enough to pass the `"type":"result"` gate)
- **When:** `render_summary(json, None)` called directly (unit test)
- **Then:** returns `Some(rendered)` (the `"type":"result"` gate is satisfied and unaffected by this defect) with the `result:` field displaying an empty string — not the partial, misleadingly-plausible-looking truncated text
- **Note:** this call site is bounded by `.unwrap_or_default()`, not `?` — so a requirement-2 violation here degrades display quality only, never gates the overall `Some`/`None` return the way IN-6's `extract_session_id()` case does; implemented as `render_summary_result_field_unterminated_falls_back_to_empty` in `tests/summary_unit_test.rs`
- **Source:** [invariant/014_json_string_extraction_escape_handling.md](../../../docs/invariant/014_json_string_extraction_escape_handling.md) § Violation Consequences, requirement 2

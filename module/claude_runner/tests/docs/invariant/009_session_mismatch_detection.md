# Test: Invariant — Session Mismatch Detection

Test case planning for [invariant/009_session_mismatch_detection.md](../../../docs/invariant/009_session_mismatch_detection.md). Tests validate that `run_print_mode()` emits a `[Runner] warning: session mismatch` diagnostic to stderr when the actual `session_id` in claude's JSON result envelope differs from the expected UUID, and that matching UUIDs, absent prior sessions, and non-JSON output all produce no warning.

**Source:** [invariant/009_session_mismatch_detection.md](../../../docs/invariant/009_session_mismatch_detection.md)
**Related:** [invariant/001_default_flags.md](../../../docs/invariant/001_default_flags.md) (`-c` injection decision), [invariant/008_render_summary_gate.md](../../../docs/invariant/008_render_summary_gate.md) (`"type":"result"` gate inherited by `extract_session_id`)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `"type":"result"` + `session_id` → `extract_session_id` returns `Some(uuid)` | Unit |
| IT-2 | `"type":"tool_use"` → `extract_session_id` returns `None` | Unit |
| IT-3 | `"type":"result"` without `session_id` → `extract_session_id` returns `None` | Unit |
| SV-1 | Fake claude emits matching UUID → no warning on stderr, exit 0 | Invariant Hold |
| SV-2 | Fake claude emits differing UUID → `[Runner] warning: session mismatch` on stderr, exit 0 | Invariant Statement |
| SV-3 | `--new-session` (no prior session, `expected_session_id=None`) → no warning regardless of binary output | Invariant Boundary |
| SV-4 | Empty session dir (`session_exists()` returns `None`) → no warning, comparison skipped | Invariant Boundary |

## Test Coverage Summary

- Unit: 3 tests (IT-1, IT-2, IT-3)
- Invariant Hold: 1 test (SV-1)
- Invariant Statement: 1 test (SV-2)
- Invariant Boundary: 2 tests (SV-3, SV-4)

**Total:** 7 invariant test cases

## Architectural Constraint

IT-1, IT-2, IT-3 are unit tests in `tests/summary_unit_test.rs` that call `extract_session_id()` directly with crafted JSON strings — no subprocess needed.

SV-1 through SV-4 are integration tests in `tests/session_verification_test.rs` using a fake `claude` binary (reusing `fake_claude_dir()` from `tests/cli_binary_test_helpers.rs`). Each test creates a separate temp storage directory with a `{UUID_A}.jsonl` file (non-empty) and passes `--session-dir <temp>` to `clr`; this makes `session_exists()` return `Some(SessionId("UUID_A"))` without any live session scanning. The fake `claude` script unconditionally prints a hardcoded CLR JSON envelope to stdout and ignores its arguments; sv1 emits UUID_A (match), sv2 emits UUID_B (mismatch). The warning block in `run_print_mode()` fires — or does not fire — based solely on the `expected_session_id` vs. `actual` comparison, not on the binary's knowledge of the test's temp dir.

## Implementation Notes

| ID | Test Function | File |
|----|---------------|------|
| IT-1 | `extract_session_id_returns_uuid_for_valid_envelope` | `tests/summary_unit_test.rs` |
| IT-2 | `extract_session_id_returns_none_for_non_result_type` | `tests/summary_unit_test.rs` |
| IT-3 | `extract_session_id_returns_none_when_session_id_absent` | `tests/summary_unit_test.rs` |
| SV-1 | `sv1_matching_uuid_emits_no_warning` | `tests/session_verification_test.rs` |
| SV-2 | `sv2_mismatched_uuid_emits_warning_but_exits_zero` | `tests/session_verification_test.rs` |
| SV-3 | `sv3_new_session_flag_skips_mismatch_check` | `tests/session_verification_test.rs` |
| SV-4 | `sv4_empty_session_dir_skips_mismatch_check` | `tests/session_verification_test.rs` |

---

### IT-1: `"type":"result"` + `session_id` → `extract_session_id` returns `Some(uuid)`

- **Given:** JSON string `{"type":"result","session_id":"abc-123","result":"hello","is_error":false}`
- **When:** `extract_session_id(json)` called directly (unit test)
- **Then:** Returns `Some("abc-123")` — `"type":"result"` guard satisfied; `session_id` field present and extracted
- **Exit:** N/A (unit test; assertion: `assert_eq!(result, Some("abc-123".to_string()))`)
- **Source:** [invariant/009_session_mismatch_detection.md](../../../docs/invariant/009_session_mismatch_detection.md) Enforcement Mechanism § summary.rs

---

### IT-2: `"type":"tool_use"` → `extract_session_id` returns `None`

- **Given:** JSON string `{"type":"tool_use","name":"bash"}`
- **When:** `extract_session_id(json)` called directly (unit test)
- **Then:** Returns `None` — `msg_type != "result"` guard fires; non-result type excluded per invariant/009 table row 4
- **Exit:** N/A (unit test; assertion: `assert_eq!(result, None)`)
- **Source:** [invariant/009_session_mismatch_detection.md](../../../docs/invariant/009_session_mismatch_detection.md) Invariant Statement table row 4

---

### IT-3: `"type":"result"` without `session_id` → `extract_session_id` returns `None`

- **Given:** JSON string `{"type":"result","result":"hello","is_error":false}` (no `session_id` field)
- **When:** `extract_session_id(json)` called directly (unit test)
- **Then:** Returns `None` — `"type":"result"` guard satisfied; `extract_str(stdout, "session_id")` returns `None` (field absent); `?` propagates `None` to caller
- **Exit:** N/A (unit test; assertion: `assert_eq!(result, None)`)
- **Source:** [invariant/009_session_mismatch_detection.md](../../../docs/invariant/009_session_mismatch_detection.md) Invariant Statement table row 4

---

### SV-1: Fake claude emits matching UUID → no warning, exit 0

- **Given:** Temp storage dir with `UUID_A.jsonl` (non-empty); `--session-dir <temp>`; fake claude emits `{"type":"result","session_id":"UUID_A","result":"hello","is_error":false}`; default `--output-style summary`
- **When:** `clr -p --max-sessions 0 --session-dir <temp> "x"` with fake claude binary in PATH
- **Then:** Exit 0; stderr does NOT contain `"session mismatch"`; `expected_session_id == actual` comparison is equal; warning block not entered
- **Exit:** 0
- **Source:** [invariant/009_session_mismatch_detection.md](../../../docs/invariant/009_session_mismatch_detection.md) Invariant Statement table row 2 (match → silent success)

---

### SV-2: Fake claude emits differing UUID → `[Runner] warning` on stderr, exit 0

- **Given:** Temp storage dir with `UUID_A.jsonl` (non-empty); `--session-dir <temp>`; fake claude emits `{"type":"result","session_id":"UUID_B","result":"hello","is_error":false}` (UUID_B ≠ UUID_A); default `--output-style summary`
- **When:** `clr -p --max-sessions 0 --session-dir <temp> "x"` with fake claude binary in PATH
- **Then:** Exit 0 (non-fatal — warning is diagnostic only); stderr contains exactly one line matching `"[Runner] warning: session mismatch — expected UUID_A, got UUID_B (BUG-320 detected)"`
- **Exit:** 0
- **Source:** [invariant/009_session_mismatch_detection.md](../../../docs/invariant/009_session_mismatch_detection.md) Invariant Statement table row 3; Warning Format section

---

### SV-3: `--new-session` → no warning, `expected_session_id=None`, exit 0

- **Given:** Fresh temp storage dir (no `.jsonl` files); `--session-dir <temp>`; `--new-session`; fake claude emits a CLR JSON envelope; default `--output-style summary`
- **When:** `clr -p --new-session --max-sessions 0 --session-dir <temp> "x"` with fake claude binary
- **Then:** Exit 0; stderr does NOT contain `"session mismatch"`; `session_exists()` returns `None`; `-c` not injected; `expected_session_id = None`; `if let Some(expected)` guard short-circuits before `extract_session_id()` is ever called
- **Exit:** 0
- **Source:** [invariant/009_session_mismatch_detection.md](../../../docs/invariant/009_session_mismatch_detection.md) Invariant Statement table row 1 (`expected_session_id` is `None` → no comparison)

---

### SV-4: Empty session dir → no warning, `expected_session_id=None`, exit 0

- **Given:** Empty temp storage dir (no `.jsonl` files); `--session-dir <temp>`; fake claude emits a CLR JSON envelope with `session_id=UUID_B`; `--output-style raw`
- **When:** `clr --max-sessions 0 --session-dir <temp> --output-style raw "x"` with fake claude binary
- **Then:** Exit 0; stderr does NOT contain `"session mismatch"`; `session_exists()` returns `None` for empty dir; `expected_session_id = None`; `if let Some(expected)` guard short-circuits before `extract_session_id()` is ever called
- **Exit:** 0
- **Source:** [invariant/009_session_mismatch_detection.md](../../../docs/invariant/009_session_mismatch_detection.md) Invariant Statement table row 1 (`expected_session_id` is `None` → no comparison)

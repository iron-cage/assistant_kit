# Test: Invariant — Session Mismatch Detection

Test case planning for [invariant/009_session_mismatch_detection.md](../../../docs/invariant/009_session_mismatch_detection.md). Tests validate that `run_print_mode()` emits a `[Runner] warning: session mismatch` diagnostic to stderr when the actual `session_id` in claude's JSON result envelope differs from the expected UUID, and that matching UUIDs, absent prior sessions, and non-JSON output all produce no warning.

**Source:** [invariant/009_session_mismatch_detection.md](../../../docs/invariant/009_session_mismatch_detection.md)
**Related:** [invariant/001_default_flags.md](../../../docs/invariant/001_default_flags.md) (`-c` injection decision), [invariant/008_render_summary_gate.md](../../../docs/invariant/008_render_summary_gate.md) (compound gate `subtype` presence OR `"type":"result"` inherited by `extract_session_id` — BUG-437 fix)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Old SDK (`"type":"result"`) + `session_id` → `extract_session_id` returns `Some(uuid)` | Unit |
| IT-2 | `"type":"tool_use"` → `extract_session_id` returns `None` | Unit |
| IT-3 | `"type":"result"` without `session_id` → `extract_session_id` returns `None` | Unit |
| IT-4 | New SDK (`subtype` present, no top-level `type`) + `session_id` → `extract_session_id` returns `Some(uuid)` (BUG-437 reproducer) | Regression Guard |
| SV-1 | Fake claude emits matching UUID → no warning on stderr, exit 0 | Invariant Hold |
| SV-2 | Fake claude emits differing UUID → `[Runner] warning: session mismatch` on stderr, exit 0 | Invariant Statement |
| SV-3 | `--new-session` (no prior session, `expected_session_id=None`) → no warning regardless of binary output | Invariant Boundary |
| SV-4 | Empty session source (`--from`, `session_exists()` returns `None`) → no warning, comparison skipped | Invariant Boundary |

## Test Coverage Summary

- Unit: 3 tests (IT-1, IT-2, IT-3)
- Regression Guard: 1 test (IT-4)
- Invariant Hold: 1 test (SV-1)
- Invariant Statement: 1 test (SV-2)
- Invariant Boundary: 2 tests (SV-3, SV-4)

**Total:** 8 invariant test cases

## Architectural Constraint

IT-1, IT-2, IT-3 are unit tests in `tests/summary_unit_test.rs` that call `extract_session_id()` directly with crafted JSON strings — no subprocess needed.

SV-1 through SV-4 are integration tests in `tests/session_verification_test.rs` using a fake `claude` binary (reusing `fake_claude_dir()` from `tests/cli_binary_test_helpers.rs`). Each test seeds a `{UUID_A}.jsonl` session file under an isolated `CLAUDE_HOME` for a source path via `make_session_for()`, then passes `--from <src>` to `clr`; this makes `session_exists()` return `Some(SessionId("UUID_A"))` via `--from`'s resolved source storage, without any live session scanning. Fix(BUG-493): `expected_session_id` (and therefore the BUG-320 mismatch check) is fed exclusively by `--from`'s resolved source storage, never by raw `--session-dir` — `--session-dir` is deprecated and inert and is explicitly excluded from these tests' environment (`env_remove("CLR_SESSION_DIR")`). The fake `claude` script unconditionally prints a hardcoded CLR JSON envelope to stdout and ignores its arguments; sv1 emits UUID_A (match), sv2 emits UUID_B (mismatch). The warning block in `run_print_mode()` fires — or does not fire — based solely on the `expected_session_id` vs. `actual` comparison, not on the binary's knowledge of the test's temp dir.

## Implementation Notes

| ID | Test Function | File |
|----|---------------|------|
| IT-1 | `extract_session_id_returns_uuid_for_valid_envelope` | `tests/summary_unit_test.rs` |
| IT-2 | `extract_session_id_returns_none_for_non_result_type` | `tests/summary_unit_test.rs` |
| IT-3 | `extract_session_id_returns_none_when_session_id_absent` | `tests/summary_unit_test.rs` |
| IT-4 | `extract_session_id_returns_uuid_for_new_sdk_envelope` | `tests/summary_unit_test.rs` |
| SV-1 | `sv1_matching_uuid_emits_no_warning` | `tests/session_verification_test.rs` |
| SV-2 | `sv2_mismatched_uuid_emits_warning_but_exits_zero` | `tests/session_verification_test.rs` |
| SV-3 | `sv3_new_session_flag_skips_mismatch_check` | `tests/session_verification_test.rs` |
| SV-4 | `sv4_empty_session_dir_skips_mismatch_check` | `tests/session_verification_test.rs` |

---

### IT-1: Old SDK (`"type":"result"`) + `session_id` → `extract_session_id` returns `Some(uuid)`

- **Given:** JSON string `{"type":"result","session_id":"abc-123","result":"hello","is_error":false}`
- **When:** `extract_session_id(json)` called directly (unit test)
- **Then:** Returns `Some("abc-123")` — compound gate passes (`"type":"result"` arm satisfied); `session_id` field present and extracted
- **Exit:** N/A (unit test; assertion: `assert_eq!(result, Some("abc-123".to_string()))`)
- **Source:** [invariant/009_session_mismatch_detection.md](../../../docs/invariant/009_session_mismatch_detection.md) Enforcement Mechanism § summary.rs

---

### IT-2: `"type":"tool_use"` → `extract_session_id` returns `None`

- **Given:** JSON string `{"type":"tool_use","name":"bash"}`
- **When:** `extract_session_id(json)` called directly (unit test)
- **Then:** Returns `None` — compound gate fires: `subtype` absent AND `msg_type == "tool_use" != "result"`; non-result type excluded per invariant/009 table row 4
- **Exit:** N/A (unit test; assertion: `assert_eq!(result, None)`)
- **Source:** [invariant/009_session_mismatch_detection.md](../../../docs/invariant/009_session_mismatch_detection.md) Invariant Statement table row 4

---

### IT-3: `"type":"result"` without `session_id` → `extract_session_id` returns `None`

- **Given:** JSON string `{"type":"result","result":"hello","is_error":false}` (no `session_id` field)
- **When:** `extract_session_id(json)` called directly (unit test)
- **Then:** Returns `None` — compound gate passes (`"type":"result"` arm satisfied); `extract_str(json, "session_id")` returns `None` (field absent); function returns `None` directly (no `?`)
- **Exit:** N/A (unit test; assertion: `assert_eq!(result, None)`)
- **Source:** [invariant/009_session_mismatch_detection.md](../../../docs/invariant/009_session_mismatch_detection.md) Invariant Statement table row 4

---

### IT-4: New SDK (`subtype` present, no top-level `type`) + `session_id` → `extract_session_id` returns `Some(uuid)` (BUG-437 reproducer)

- **Given:** JSON string `{"subtype":"success","session_id":"abc-123","usage":{"iterations":[{"type":"message"}]}}` — new SDK envelope format with no top-level `"type"` field; `extract_str(json,"type")` finds nested `"message"` first
- **When:** `extract_session_id(json)` called directly (unit test)
- **Then:** Returns `Some("abc-123")` — compound gate passes via `subtype` arm (`subtype.is_some()` = true); the old `"type":"result"`-only gate would have fired (found `"message"` from iterations) and incorrectly returned `None`; BUG-320 session mismatch detection is preserved for new SDK envelopes
- **Exit:** N/A (unit test; assertion: `assert_eq!(result, Some("abc-123".to_string()))`)
- **Source:** [invariant/009_session_mismatch_detection.md](../../../docs/invariant/009_session_mismatch_detection.md) Enforcement Mechanism § summary.rs; BUG-437 regression coverage

---

### SV-1: Fake claude emits matching UUID → no warning, exit 0

- **Given:** `UUID_A.jsonl` session seeded under an isolated `CLAUDE_HOME` for source path `src` (via `make_session_for()`); `--from <src>`; fake claude emits `{"type":"result","session_id":"UUID_A","result":"hello","is_error":false}`; default `--output-style summary`
- **When:** `clr -p --max-sessions 0 --from <src> "x"` with fake claude binary in PATH
- **Then:** Exit 0; stderr does NOT contain `"session mismatch"`; `expected_session_id == actual` comparison is equal; warning block not entered
- **Exit:** 0
- **Source:** [invariant/009_session_mismatch_detection.md](../../../docs/invariant/009_session_mismatch_detection.md) Invariant Statement table row 2 (match → silent success)

---

### SV-2: Fake claude emits differing UUID → `[Runner] warning` on stderr, exit 0

- **Given:** `UUID_A.jsonl` session seeded under an isolated `CLAUDE_HOME` for source path `src`; `--from <src>`; fake claude emits `{"type":"result","session_id":"UUID_B","result":"hello","is_error":false}` (UUID_B ≠ UUID_A); default `--output-style summary`
- **When:** `clr -p --max-sessions 0 --from <src> "x"` with fake claude binary in PATH
- **Then:** Exit 0 (non-fatal — warning is diagnostic only); stderr contains exactly one line matching `"[Runner] warning: session mismatch — expected UUID_A, got UUID_B (BUG-320 detected)"`
- **Exit:** 0
- **Source:** [invariant/009_session_mismatch_detection.md](../../../docs/invariant/009_session_mismatch_detection.md) Invariant Statement table row 3; Warning Format section

---

### SV-3: `--new-session` → no warning, `expected_session_id=None`, exit 0

- **Given:** `UUID_A.jsonl` session seeded under an isolated `CLAUDE_HOME` for source path `src`; `--from <src>`; `--new-session`; fake claude emits a CLR JSON envelope; default `--output-style summary`
- **When:** `clr -p --new-session --max-sessions 0 --from <src> "x"` with fake claude binary
- **Then:** Exit 0; stderr does NOT contain `"session mismatch"`; `session_exists()` returns `None`; `-c` not injected; `expected_session_id = None`; `if let Some(expected)` guard short-circuits before `extract_session_id()` is ever called
- **Exit:** 0
- **Source:** [invariant/009_session_mismatch_detection.md](../../../docs/invariant/009_session_mismatch_detection.md) Invariant Statement table row 1 (`expected_session_id` is `None` → no comparison)

---

### SV-4: Empty session source → no warning, `expected_session_id=None`, exit 0

- **Given:** Source path `src` with no session seeded (never called `make_session_for()`); `--from <src>`; fake claude emits a CLR JSON envelope with `session_id=UUID_B`; `--output-style raw`
- **When:** `clr --max-sessions 0 --from <src> --output-style raw "x"` with fake claude binary
- **Then:** Exit 0; stderr does NOT contain `"session mismatch"`; `session_exists()` returns `None` for the unseeded source; `expected_session_id = None`; `if let Some(expected)` guard short-circuits before `extract_session_id()` is ever called
- **Exit:** 0
- **Source:** [invariant/009_session_mismatch_detection.md](../../../docs/invariant/009_session_mismatch_detection.md) Invariant Statement table row 1 (`expected_session_id` is `None` → no comparison)

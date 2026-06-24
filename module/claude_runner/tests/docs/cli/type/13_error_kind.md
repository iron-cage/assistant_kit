# Type :: `ErrorKind`

Validation tests for the `ErrorKind` classification type. Tests validate subprocess error classification logic and CLR-layer exit codes.

**Source:** [type/13_error_kind.md](../../../../docs/cli/type/13_error_kind.md)
**Invariant:** [invariant/006_exit_codes.md](../../../../docs/invariant/006_exit_codes.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Exit 0 → `None` | Success |
| TC-2 | Exit 2, no text → `RateLimit` | Transient |
| TC-3 | Exit 2 + `"You've hit your limit"` → `QuotaExhausted` | Account |
| TC-4 | `"Your organization does not have access to Claude"` → `AuthError` | Auth |
| TC-5 | `"API Error: "` → `ApiError` | Service |
| TC-6 | Exit > 128 → `Signal` | Process |
| TC-7 | Exit 1, no pattern → `Unknown` | Unknown |
| TC-8 | `QuotaExhausted` takes priority over exit-2 `RateLimit` | Priority Order |
| TC-9 | `AuthError` takes priority over `ApiError` | Priority Order |
| TC-10 | CLR timeout → exit 4, stderr `"Error: timeout after {N}s"` | CLR-Layer |
| TC-11 | `--expect` mismatch → exit 3 | CLR-Layer |
| TC-12 | `--max-sessions 0`, no gate → still exits 0 on success | CLR-Layer |
| TC-13 | `authentication_error` 401 string → `AuthError`, not `ApiError` | Auth / Priority Order |

## Test Coverage Summary

- Success: 1 test (TC-1)
- Transient: 1 test (TC-2)
- Account: 1 test (TC-3)
- Auth: 1 test (TC-4)
- Service: 1 test (TC-5)
- Process: 1 test (TC-6)
- Unknown: 1 test (TC-7)
- Priority Order: 2 tests (TC-8, TC-9)
- CLR-Layer: 3 tests (TC-10, TC-11, TC-12)
- Auth / Priority Order: 1 test (TC-13)

**Total:** 13 test cases

## Test Cases

---

### TC-1: Exit 0 → `None`

- **Given:** `ExecutionOutput` with `exit_code = 0`
- **When:** `classify_error()` called
- **Then:** Returns `None`
- **Exit:** N/A (unit test)
- **Source:** [type/13_error_kind.md](../../../../docs/cli/type/13_error_kind.md)

---

### TC-2: Exit 2, no text → `RateLimit`

- **Given:** `ExecutionOutput` with `exit_code = 2`, empty stdout and stderr
- **When:** `classify_error()` called
- **Then:** Returns `Some(ErrorKind::RateLimit)`
- **Exit:** N/A (unit test)
- **Source:** [type/13_error_kind.md](../../../../docs/cli/type/13_error_kind.md)

---

### TC-3: Exit 2 + quota text → `QuotaExhausted`

- **Given:** `ExecutionOutput` with `exit_code = 2`, stdout contains `"You've hit your limit"`
- **When:** `classify_error()` called
- **Then:** Returns `Some(ErrorKind::QuotaExhausted)` (not `RateLimit`)
- **Exit:** N/A (unit test)
- **Source:** [type/13_error_kind.md](../../../../docs/cli/type/13_error_kind.md)

---

### TC-4: Auth text → `AuthError`

- **Given:** `ExecutionOutput` with `exit_code = 1`, stderr contains `"Your organization does not have access to Claude"`
- **When:** `classify_error()` called
- **Then:** Returns `Some(ErrorKind::AuthError)`
- **Exit:** N/A (unit test)
- **Source:** [type/13_error_kind.md](../../../../docs/cli/type/13_error_kind.md)

---

### TC-5: API error text → `ApiError`

- **Given:** `ExecutionOutput` with `exit_code = 1`, stderr contains `"API Error: 500 Internal Server Error"`
- **When:** `classify_error()` called
- **Then:** Returns `Some(ErrorKind::ApiError)`
- **Exit:** N/A (unit test)
- **Source:** [type/13_error_kind.md](../../../../docs/cli/type/13_error_kind.md)

---

### TC-6: Exit > 128 → `Signal`

- **Given:** `ExecutionOutput` with `exit_code = 143` (128 + SIGTERM 15)
- **When:** `classify_error()` called
- **Then:** Returns `Some(ErrorKind::Signal)`
- **Exit:** N/A (unit test)
- **Source:** [type/13_error_kind.md](../../../../docs/cli/type/13_error_kind.md)

---

### TC-7: Exit 1, no pattern → `Unknown`

- **Given:** `ExecutionOutput` with `exit_code = 1`, empty stdout/stderr
- **When:** `classify_error()` called
- **Then:** Returns `Some(ErrorKind::Unknown)`
- **Exit:** N/A (unit test)
- **Source:** [type/13_error_kind.md](../../../../docs/cli/type/13_error_kind.md)

---

### TC-8: `QuotaExhausted` priority over `RateLimit` sentinel

- **Given:** `ExecutionOutput` with `exit_code = 2`, stdout contains `"You've hit your limit"`
- **When:** `classify_error()` called
- **Then:** Returns `Some(ErrorKind::QuotaExhausted)`, not `Some(ErrorKind::RateLimit)`
- **Exit:** N/A (unit test)
- **Source:** [type/13_error_kind.md](../../../../docs/cli/type/13_error_kind.md)

---

### TC-9: `AuthError` priority over `ApiError`

- **Given:** `ExecutionOutput` with `exit_code = 1`, stderr contains both `"Your organization does not have access to Claude"` and `"API Error: "`
- **When:** `classify_error()` called
- **Then:** Returns `Some(ErrorKind::AuthError)`, not `Some(ErrorKind::ApiError)`
- **Exit:** N/A (unit test)
- **Source:** [type/13_error_kind.md](../../../../docs/cli/type/13_error_kind.md)

---

### TC-10: CLR timeout → exit 4 with stderr label

- **Given:** fake `claude` process running; `clr --timeout 1 "msg"` with 1-second timeout
- **When:** subprocess does not exit within 1 second
- **Then:** `clr` exits 4; stderr contains `"Error: timeout after 1s"`
- **Exit:** 4
- **Platform:** requires fake process that sleeps longer than timeout
- **Source:** [type/13_error_kind.md](../../../../docs/cli/type/13_error_kind.md)

---

### TC-11: `--expect` mismatch → exit 3

- **Given:** fake `claude` that prints `"foo"` (does not match `--expect "bar"`)
- **When:** `clr --print --expect "bar" "msg"`
- **Then:** `clr` exits 3; stderr contains expect-mismatch message
- **Exit:** 3
- **Source:** [type/13_error_kind.md](../../../../docs/cli/type/13_error_kind.md)

---

### TC-12: `--max-sessions 0` bypasses gate; exits 0 on success

- **Given:** fake `claude` that exits 0; `clr --max-sessions 0 --print "msg"`
- **When:** invoked
- **Then:** exits 0 (gate is disabled; no exit 1 from gate timeout)
- **Exit:** 0
- **Source:** [type/13_error_kind.md](../../../../docs/cli/type/13_error_kind.md)

---

### TC-13: `authentication_error` 401 string → `AuthError`, not `ApiError`

- **Given:** `ExecutionOutput` with `exit_code = 1`, stderr contains `"Failed to authenticate. API Error: 401 {\"type\":\"authentication_error\",\"message\":\"Invalid authentication credentials\"}"`
- **When:** `classify_error()` called
- **Then:** Returns `Some(ErrorKind::AuthError)`, not `Some(ErrorKind::ApiError)` — the `"authentication_error"` pattern fires before the `"API Error: "` catch-all
- **Exit:** N/A (unit test)
- **Note:** test_kind: bug_reproducer(BUG-314). Without the fix the string would be misclassified as `ApiError` because `"API Error: "` appears in the same message.
- **Source:** [type/13_error_kind.md](../../../../docs/cli/type/13_error_kind.md)

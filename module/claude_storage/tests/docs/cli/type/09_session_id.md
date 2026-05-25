# Type :: 9. `SessionId`

Type constraint tests for `SessionId` — exact session identifier.

**Source:** [type/09_session_id.md](../../../../docs/cli/type/09_session_id.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Named format accepted (dash-prefixed) | Valid Input |
| TC-2 | UUID format accepted | Valid Input |
| TC-3 | Empty input rejected | Invalid Input |
| TC-4 | Unresolvable session rejected | Not Found |
| TC-5 | Named vs UUID detection | Format Detection |

## Test Coverage Summary

- Valid Input: 2 tests (TC-1, TC-2)
- Invalid Input: 1 test (TC-3)
- Not Found: 1 test (TC-4)
- Format Detection: 1 test (TC-5)

**Total:** 5 cases

## Test Cases

---

### TC-1: Named format accepted (dash-prefixed)

- **Given:** Input string `"-default_topic"` and the session file exists in storage
- **When:** `SessionId` is parsed and resolved
- **Then:** Accepted; `is_named()` returns true; `filename()` returns `"-default_topic.jsonl"`

---

### TC-2: UUID format accepted

- **Given:** Input string `"8d795a1c-c81d-4010-8d29-b4e678272419"` and session file exists
- **When:** `SessionId` is parsed and resolved
- **Then:** Accepted; `is_uuid()` returns true; `filename()` returns `"8d795a1c-c81d-4010-8d29-b4e678272419.jsonl"`

---

### TC-3: Empty input rejected

- **Given:** Input string `""`
- **When:** `SessionId` is parsed
- **Then:** Rejected; error message is `session_id must be non-empty`

---

### TC-4: Unresolvable session rejected

- **Given:** Input string `"-no-such-session"` and no matching `.jsonl` file in storage
- **When:** `SessionId` is resolved
- **Then:** Rejected; error message is `session not found: -no-such-session`

---

### TC-5: Named vs UUID detection

- **Given:** Input string `"abc123"` (no leading `-`, not UUID pattern)
- **When:** `SessionId` is parsed and resolved (session exists)
- **Then:** Accepted; `is_named()` returns false; `is_uuid()` returns false; `filename()` returns `"abc123.jsonl"`

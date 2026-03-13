# Parameter :: `session_id::`

Edge case tests for the `session_id::` parameter (direct identifier, not filter). Tests validate required enforcement and not-found handling.

**Source:** [params.md#parameter--14-session_id](../../params.md#parameter--14-session_id) | [types.md#sessionid](../../types.md#sessionid)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Named session ID (e.g., -default_topic) accepted | Format |
| EC-2 | UUID session ID accepted | Format |
| EC-3 | Empty value rejected | Boundary Values |
| EC-4 | Unknown session ID exits with error | Error Handling |
| EC-5 | Required in .export — missing exits with 1 | Required Enforcement |
| EC-6 | Optional in .show — absent shows project | Optional Behavior |
| EC-7 | Whitespace-only value rejected | Boundary Values |

## Test Coverage Summary

- Format: 2 tests (EC-1, EC-2)
- Boundary Values: 2 tests (EC-3, EC-7)
- Error Handling: 1 test (EC-4)
- Required Enforcement: 1 test (EC-5)
- Optional Behavior: 1 test (EC-6)

## Test Cases

### EC-1: Named session ID (e.g., -default_topic) accepted

**Goal:** Verify that a human-readable named session ID (filename stem starting with `-`) is accepted and resolves the correct session.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .show session_id::-default_topic`
**Expected Output:** Session content for the session stored as `-default_topic.jsonl` in the current project.
**Verification:**
- Command exits with code 0
- Output displays conversation content from the `-default_topic` session
- No error about unrecognized format appears on stderr
**Pass Criteria:** exit 0 + content from `-default_topic` session displayed
**Source:** [params.md](../../params.md)

---

### EC-2: UUID session ID accepted

**Goal:** Verify that a UUID-format session ID is accepted and resolves the correct session.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .show session_id::8d795a1c-c81d-4010-8d29-b4e678272419`
**Expected Output:** Session content for the session stored as `8d795a1c-c81d-4010-8d29-b4e678272419.jsonl`.
**Verification:**
- Command exits with code 0
- Output displays conversation content from the UUID-named session
- No error about format appears on stderr
**Pass Criteria:** exit 0 + content from UUID session displayed
**Source:** [params.md](../../params.md)

---

### EC-3: Empty value rejected

**Goal:** Verify that an empty `session_id::` value is rejected before any storage lookup.
**Setup:** None
**Command:** `clg .show session_id::`
**Expected Output:** Error about empty session ID value (e.g., `session_id must be non-empty`).
**Verification:**
- Command exits with code 1
- Stderr contains an error message about the empty value
- No storage lookup is attempted
**Pass Criteria:** exit 1 + error about empty session_id value
**Source:** [params.md](../../params.md)

---

### EC-4: Unknown session ID exits with error

**Goal:** Verify that a session ID that doesn't exist in storage produces the exact not-found error message.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .show session_id::-nonexistent-session-zzz`
**Expected Output:** `session not found: -nonexistent-session-zzz`
**Verification:**
- Command exits with code 1
- Stderr contains the string `session not found: -nonexistent-session-zzz`
**Pass Criteria:** exit 1 + error message `session not found: -nonexistent-session-zzz`
**Source:** [params.md](../../params.md)

---

### EC-5: Required in .export — missing exits with 1

**Goal:** Verify that running `.export` without `session_id::` exits with an error because the parameter is required for that command.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .export output::/tmp/out.md`
**Expected Output:** Error indicating `session_id::` is required for `.export`.
**Verification:**
- Command exits with code 1
- Stderr contains an error message about the missing required `session_id::` parameter
- No output file is created at `/tmp/out.md`
**Pass Criteria:** exit 1 + error about missing `session_id::` for `.export`
**Source:** [params.md](../../params.md)

---

### EC-6: Optional in .show — absent shows project

**Goal:** Verify that omitting `session_id::` from `.show` shows the project view (not an error).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .show` (run from a directory with a known project in the fixture)
**Expected Output:** Project-level view (list of sessions, project metadata) rather than a single session's content.
**Verification:**
- Command exits with code 0
- Output shows project-level information (multiple sessions listed) rather than a single session content view
- No error about missing `session_id::` appears on stderr
**Pass Criteria:** exit 0 + project view shown (not a single-session view, not an error)
**Source:** [params.md](../../params.md)

---

### EC-7: Whitespace-only value rejected

**Goal:** Verify that a whitespace-only string is rejected as an invalid session ID (treated as empty/non-empty boundary).
**Setup:** None
**Command:** `clg .show session_id::   ` (value is spaces only)
**Expected Output:** Error about invalid or empty session ID value.
**Verification:**
- Command exits with code 1
- Stderr contains an error message indicating the session ID is invalid or empty
**Pass Criteria:** exit 1 + error about whitespace-only session_id value
**Source:** [params.md](../../params.md)

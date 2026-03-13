# Parameter :: `session::`

Edge case tests for the `session::` parameter (filter, not `session_id::`). Tests validate substring matching and auto-enable behavior.

**Source:** [params.md#parameter--13-session](../../params.md#parameter--13-session) | [types.md#sessionfilter](../../types.md#sessionfilter)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Partial match at start of session ID | Matching |
| EC-2 | Partial match in middle of session ID | Matching |
| EC-3 | Case-insensitive match | Matching |
| EC-4 | No match returns empty results | Matching |
| EC-5 | Empty value rejected | Boundary Values |
| EC-6 | Auto-enables sessions::1 in .list | Auto-Enable |
| EC-7 | session:: in .count restricts to matching session | Scoping |

## Test Coverage Summary

- Matching: 4 tests (EC-1, EC-2, EC-3, EC-4)
- Boundary Values: 1 test (EC-5)
- Auto-Enable: 1 test (EC-6)
- Scoping: 1 test (EC-7)

## Test Cases

### EC-1: Partial match at start of session ID

**Goal:** Verify that a substring matching the beginning of a session filename stem returns that session.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .sessions session::default`
**Expected Output:** Sessions whose ID starts with `default` (e.g., `-default_topic`) appear in results.
**Verification:**
- Command exits with code 0
- Session `-default_topic` (or equivalent) appears in output
- Sessions not matching `default` are excluded
**Pass Criteria:** exit 0 + sessions starting with `default` in their ID are returned
**Source:** [params.md](../../params.md)

---

### EC-2: Partial match in middle of session ID

**Goal:** Verify that a substring matching the middle of a session filename stem returns that session.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .sessions session::topic`
**Expected Output:** Sessions containing `topic` anywhere in their ID (e.g., `-default_topic`) appear in results.
**Verification:**
- Command exits with code 0
- Session `-default_topic` (or equivalent containing `topic`) appears in output
- Sessions without `topic` in their ID are excluded
**Pass Criteria:** exit 0 + sessions containing `topic` in their ID are returned
**Source:** [params.md](../../params.md)

---

### EC-3: Case-insensitive match

**Goal:** Verify that `session::` matching is case-insensitive (uppercase filter matches lowercase session ID).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .sessions session::DEFAULT`
**Expected Output:** Same sessions returned as `session::default` — case difference is ignored.
**Verification:**
- Command exits with code 0
- Sessions matching `default` (lowercase) in their ID appear in output
- Result set is identical to `session::default` result set
**Pass Criteria:** exit 0 + same results as lowercase equivalent filter
**Source:** [params.md](../../params.md)

---

### EC-4: No match returns empty results

**Goal:** Verify that a non-matching filter returns an empty list without an error.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .sessions session::zzznomatch999`
**Expected Output:** Empty session list or "no sessions found" message; no error exit code.
**Verification:**
- Command exits with code 0
- No sessions appear in output
- No error about the filter value appears on stderr
**Pass Criteria:** exit 0 + empty result set (no error for non-matching filter)
**Source:** [params.md](../../params.md)

---

### EC-5: Empty value rejected

**Goal:** Verify that an empty `session::` value is rejected with an error.
**Setup:** None
**Command:** `clg .sessions session::`
**Expected Output:** Error about empty session filter value (e.g., `session filter must be non-empty`).
**Verification:**
- Command exits with code 1
- Stderr contains an error message about the empty filter value
**Pass Criteria:** exit 1 + error message about empty session filter
**Source:** [params.md](../../params.md)

---

### EC-6: Auto-enables sessions::1 in .list

**Goal:** Verify that providing `session::` in `.list` automatically enables session display without requiring explicit `sessions::1`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .list session::default`
**Expected Output:** Project list with session rows shown under each project; only sessions matching `default` are shown.
**Verification:**
- Command exits with code 0
- Session rows appear in output under projects (not just project-level summary rows)
- Only sessions with `default` in their ID are shown
**Pass Criteria:** exit 0 + session display auto-enabled and filtered by `default` substring
**Source:** [params.md](../../params.md)

---

### EC-7: session:: in .count restricts to matching session

**Goal:** Verify that `session::` in `.count` restricts the count operation to only matching sessions.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .count target::entries session::default`
**Expected Output:** Entry count only for sessions matching `default`; not a total across all sessions.
**Verification:**
- Command exits with code 0
- Returned count reflects only entries from sessions whose ID contains `default`
- Count is lower than or equal to the total entry count without the filter
**Pass Criteria:** exit 0 + count scoped to matching sessions only
**Source:** [params.md](../../params.md)

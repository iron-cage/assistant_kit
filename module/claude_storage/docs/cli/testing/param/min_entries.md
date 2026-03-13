# Parameter :: `min_entries::`

Edge case tests for the `min_entries::` parameter. Tests validate non-negative integer enforcement and auto-enable behavior.

**Source:** [params.md#parameter--7-min_entries](../../params.md#parameter--7-min_entries) | [types.md#entrycount](../../types.md#entrycount)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Value 0 accepted (no minimum) | Boundary Values |
| EC-2 | Value 1 accepted | Boundary Values |
| EC-3 | Large value (e.g., 10000) accepted | Boundary Values |
| EC-4 | Negative value rejected | Boundary Values |
| EC-5 | Float value rejected | Type Validation |
| EC-6 | String "ten" rejected | Type Validation |
| EC-7 | Auto-enables sessions display in .list | Auto-Enable |
| EC-8 | Unset shows all sessions (no threshold) | Default |

## Test Coverage Summary

- Boundary Values: 4 tests (EC-1, EC-2, EC-3, EC-4)
- Type Validation: 2 tests (EC-5, EC-6)
- Auto-Enable: 1 test (EC-7)
- Default: 1 test (EC-8)

## Test Cases

### EC-1: Value 0 accepted (no minimum)

**Goal:** Verify that `min_entries::0` is accepted and imposes no threshold (all sessions included).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .sessions min_entries::0`
**Expected Output:** All sessions listed regardless of entry count, including sessions with very few entries.
**Verification:**
- Command exits with code 0
- No error message appears on stderr
- Sessions with only 1 or 2 entries are included in output
**Pass Criteria:** exit 0 + result set matches the unfiltered session count for the fixture
**Source:** [params.md](../../params.md)

---

### EC-2: Value 1 accepted

**Goal:** Verify that `min_entries::1` is accepted and filters out sessions with zero entries.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .sessions min_entries::1`
**Expected Output:** Sessions with at least 1 entry listed; empty sessions excluded.
**Verification:**
- Command exits with code 0
- No error message appears on stderr
- All returned sessions have at least 1 entry
**Pass Criteria:** exit 0 + only sessions with ≥ 1 entry appear in output
**Source:** [params.md](../../params.md)

---

### EC-3: Large value (e.g., 10000) accepted

**Goal:** Verify that a large threshold value like `10000` is accepted without error (even if it matches no sessions).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .sessions min_entries::10000`
**Expected Output:** Empty session list (no sessions in the fixture have 10000+ entries).
**Verification:**
- Command exits with code 0
- No error message about out-of-range value appears on stderr
- Output is empty or shows "no sessions found" style message
**Pass Criteria:** exit 0 + empty result (large threshold accepted, no sessions match)
**Source:** [params.md](../../params.md)

---

### EC-4: Negative value rejected

**Goal:** Verify that a negative value for `min_entries::` is rejected with the exact error message.
**Setup:** None
**Command:** `clg .sessions min_entries::-1`
**Expected Output:** `min_entries must be ≥ 0, got -1`
**Verification:**
- Command exits with code 1
- Stderr contains the string `min_entries must be ≥ 0, got -1`
**Pass Criteria:** exit 1 + error message `min_entries must be ≥ 0, got -1`
**Source:** [params.md](../../params.md)

---

### EC-5: Float value rejected

**Goal:** Verify that a float value is rejected as a non-integer for `min_entries::`.
**Setup:** None
**Command:** `clg .sessions min_entries::2.5`
**Expected Output:** `min_entries must be a non-negative integer, got 2.5`
**Verification:**
- Command exits with code 1
- Stderr contains the string `min_entries must be a non-negative integer, got 2.5`
**Pass Criteria:** exit 1 + error message `min_entries must be a non-negative integer, got 2.5`
**Source:** [params.md](../../params.md)

---

### EC-6: String "ten" rejected

**Goal:** Verify that a non-numeric string is rejected as a non-integer for `min_entries::`.
**Setup:** None
**Command:** `clg .sessions min_entries::ten`
**Expected Output:** `min_entries must be a non-negative integer, got ten`
**Verification:**
- Command exits with code 1
- Stderr contains the string `min_entries must be a non-negative integer, got ten`
**Pass Criteria:** exit 1 + error message `min_entries must be a non-negative integer, got ten`
**Source:** [params.md](../../params.md)

---

### EC-7: Auto-enables sessions display in .list

**Goal:** Verify that `min_entries::` in `.list` automatically enables session display without requiring explicit `sessions::1`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .list min_entries::2`
**Expected Output:** Project list with per-project session rows shown; only sessions with ≥ 2 entries appear.
**Verification:**
- Command exits with code 0
- Output includes session-level entries under projects (sessions auto-displayed)
- Any visible sessions have at least 2 entries
**Pass Criteria:** exit 0 + sessions section visible in output (auto-enabled by `min_entries::2`)
**Source:** [params.md](../../params.md)

---

### EC-8: Unset shows all sessions (no threshold)

**Goal:** Verify that omitting `min_entries::` returns all sessions regardless of entry count.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .sessions`
**Expected Output:** All sessions listed without any entry-count filter applied.
**Verification:**
- Command exits with code 0
- Sessions with very few entries (e.g., 1 entry) are not filtered out
- Result set is equal to or larger than any filtered result from `min_entries::N` with N > 0
**Pass Criteria:** exit 0 + all sessions in fixture are included (no implicit threshold applied)
**Source:** [params.md](../../params.md)

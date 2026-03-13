# Parameter :: `strategy::`

Edge case tests for the `strategy::` parameter. Tests validate enum values, case insensitivity, forcing behavior, and default (auto-detect) behavior.

**Source:** [params.md#parameter--20-strategy](../../params.md#parameter--20-strategy) | [types.md#strategytype](../../types.md#strategytype)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | resume accepted | Type Validation |
| EC-2 | fresh accepted | Type Validation |
| EC-3 | Invalid value rejected | Boundary Values |
| EC-4 | Case-insensitive: Resume accepted | Type Validation |
| EC-5 | Case-insensitive: FRESH accepted | Type Validation |
| EC-6 | Absent defaults to auto-detect (fresh when no history) | Default |
| EC-7 | Absent defaults to auto-detect (resume when history exists) | Default |
| EC-8 | resume forced overrides auto-detect fresh | Override |
| EC-9 | fresh forced overrides auto-detect resume | Override |

## Test Coverage Summary

- Type Validation: 4 tests (EC-1, EC-2, EC-4, EC-5)
- Boundary Values: 1 test (EC-3)
- Default: 2 tests (EC-6, EC-7)
- Override: 2 tests (EC-8, EC-9)

## Test Cases

### EC-1: resume accepted

**Goal:** Verify that `strategy::resume` is accepted as a valid value.
**Setup:** TempDir as HOME; base directory accessible.
**Command:** `clg .session.ensure path::{base} strategy::resume`
**Expected Output:** Two lines; line 2 is `resume`; exit code 0.
**Verification:**
- Exit code is `0`
- Line 2 is `resume`
- No error on stderr
**Pass Criteria:** exit 0 + `resume` accepted

**Source:** [params.md](../../params.md)

---

### EC-2: fresh accepted

**Goal:** Verify that `strategy::fresh` is accepted as a valid value.
**Setup:** TempDir as HOME; base directory accessible.
**Command:** `clg .session.ensure path::{base} strategy::fresh`
**Expected Output:** Two lines; line 2 is `fresh`; exit code 0.
**Verification:**
- Exit code is `0`
- Line 2 is `fresh`
- No error on stderr
**Pass Criteria:** exit 0 + `fresh` accepted

**Source:** [params.md](../../params.md)

---

### EC-3: Invalid value rejected

**Goal:** Verify that an unsupported strategy value returns an error.
**Setup:** None specific.
**Command:** `clg .session.ensure path::{base} strategy::auto`
**Expected Output:** Error message containing `"strategy must be resume|fresh"`; exit code 1.
**Verification:**
- `$?` is `1`
- stderr contains `"strategy must be resume|fresh"`
**Pass Criteria:** exit 1 + correct error message

**Source:** [params.md](../../params.md)

---

### EC-4: Case-insensitive: Resume accepted

**Goal:** Verify that `strategy::Resume` (mixed case) is accepted.
**Setup:** TempDir as HOME.
**Command:** `clg .session.ensure path::{base} strategy::Resume`
**Expected Output:** Two lines; line 2 is `resume` (normalized to lowercase); exit code 0.
**Verification:**
- Exit code is `0`
- No error on stderr
**Pass Criteria:** exit 0 + mixed-case strategy accepted

**Source:** [params.md](../../params.md)

---

### EC-5: Case-insensitive: FRESH accepted

**Goal:** Verify that `strategy::FRESH` (uppercase) is accepted.
**Setup:** TempDir as HOME.
**Command:** `clg .session.ensure path::{base} strategy::FRESH`
**Expected Output:** Two lines; line 2 is `fresh`; exit code 0.
**Verification:**
- Exit code is `0`
- No error on stderr
**Pass Criteria:** exit 0 + uppercase strategy accepted

**Source:** [params.md](../../params.md)

---

### EC-6: Absent defaults to auto-detect (fresh when no history)

**Goal:** Verify that when `strategy::` is absent and no history exists, `fresh` is reported.
**Setup:** TempDir as HOME with NO matching storage for the session directory.
**Command:** `clg .session.ensure path::{base}`
**Expected Output:** Line 2 is `fresh`.
**Verification:**
- Line 2 is `fresh`
**Pass Criteria:** line 2 is "fresh" when no history exists and strategy not forced

**Source:** [params.md](../../params.md)

---

### EC-7: Absent defaults to auto-detect (resume when history exists)

**Goal:** Verify that when `strategy::` is absent and history exists, `resume` is reported.
**Setup:** TempDir as HOME; create `~/.claude/projects/{encoded_session_dir}/` with a non-empty `.jsonl` file.
**Command:** `clg .session.ensure path::{base} topic::{topic}`
**Expected Output:** Line 2 is `resume`.
**Verification:**
- Line 2 is `resume`
**Pass Criteria:** line 2 is "resume" when history exists and strategy not forced

**Source:** [params.md](../../params.md)

---

### EC-8: resume forced overrides auto-detect fresh

**Goal:** Verify `strategy::resume` forces `resume` output even when auto-detect would give `fresh`.
**Setup:** TempDir as HOME with NO matching storage (auto-detect would be `fresh`).
**Command:** `clg .session.ensure path::{base} strategy::resume`
**Expected Output:** Line 2 is `resume` (not `fresh`).
**Verification:**
- Line 2 is `resume`
**Pass Criteria:** line 2 is "resume" despite auto-detect being "fresh"

**Source:** [params.md](../../params.md)

---

### EC-9: fresh forced overrides auto-detect resume

**Goal:** Verify `strategy::fresh` forces `fresh` output even when auto-detect would give `resume`.
**Setup:** TempDir as HOME; create storage history (auto-detect would be `resume`).
**Command:** `clg .session.ensure path::{base} topic::{topic} strategy::fresh`
**Expected Output:** Line 2 is `fresh` (not `resume`).
**Verification:**
- Line 2 is `fresh`
**Pass Criteria:** line 2 is "fresh" despite existing history

**Source:** [params.md](../../params.md)

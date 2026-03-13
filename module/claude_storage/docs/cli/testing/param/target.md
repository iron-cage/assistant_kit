# Parameter :: `target::`

Edge case tests for the `target::` parameter. Tests validate enum parsing and hierarchy requirements.

**Source:** [params.md#parameter--16-target](../../params.md#parameter--16-target) | [types.md#targettype](../../types.md#targettype)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Value "projects" accepted | Enum Values |
| EC-2 | Value "sessions" accepted | Enum Values |
| EC-3 | Value "entries" accepted | Enum Values |
| EC-4 | Value "SESSIONS" accepted (case-insensitive) | Case Insensitivity |
| EC-5 | Invalid value "files" rejected with error | Error Handling |
| EC-6 | Omitted defaults to "projects" | Default |
| EC-7 | target::sessions without project:: counts all sessions | Behavior |

## Test Coverage Summary

- Enum Values: 3 tests (EC-1, EC-2, EC-3)
- Case Insensitivity: 1 test (EC-4)
- Error Handling: 1 test (EC-5)
- Default: 1 test (EC-6)
- Behavior: 1 test (EC-7)

## Test Cases

### EC-1: Value "projects" accepted

**Goal:** Verify that `target::projects` is accepted and `.count` returns a count of all projects in storage.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .count target::projects`
**Expected Output:** stdout contains a non-negative integer representing the number of projects.
**Verification:**
- Exit code is 0
- stdout contains a numeric value (integer ≥ 0)
**Pass Criteria:** exit 0 + numeric output representing project count
**Source:** [params.md](../../params.md)

### EC-2: Value "sessions" accepted

**Goal:** Verify that `target::sessions` is accepted and `.count` returns a count of sessions in a project.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .count target::sessions`
**Expected Output:** stdout contains a non-negative integer representing the number of sessions.
**Verification:**
- Exit code is 0
- stdout contains a numeric value (integer ≥ 0)
**Pass Criteria:** exit 0 + numeric output representing session count
**Source:** [params.md](../../params.md)

### EC-3: Value "entries" accepted

**Goal:** Verify that `target::entries` is accepted and `.count` returns a count of entries in a session.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .count target::entries session_id::-default_topic`
**Expected Output:** stdout contains a non-negative integer representing the number of entries.
**Verification:**
- Exit code is 0
- stdout contains a numeric value (integer ≥ 0)
**Pass Criteria:** exit 0 + numeric output representing entry count
**Source:** [params.md](../../params.md)

### EC-4: Value "SESSIONS" accepted (case-insensitive)

**Goal:** Verify that enum parsing is case-insensitive and `target::SESSIONS` is treated identically to `target::sessions`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .count target::SESSIONS`
**Expected Output:** No error; output is identical to using lowercase `target::sessions`.
**Verification:**
- Exit code is 0
- stdout contains a numeric value (same result as EC-2)
**Pass Criteria:** exit 0 + numeric output (case normalization applied)
**Source:** [params.md](../../params.md)

### EC-5: Invalid value "files" rejected with error

**Goal:** Verify that `target::files` is rejected with the exact error message `"target must be projects|sessions|entries, got files"`.
**Setup:** None
**Command:** `clg .count target::files`
**Expected Output:** stderr contains `target must be projects|sessions|entries, got files`
**Verification:**
- Exit code is 1
- stderr contains the exact string `target must be projects|sessions|entries, got files`
**Pass Criteria:** exit 1 + error message `target must be projects|sessions|entries, got files`
**Source:** [params.md](../../params.md)

### EC-6: Omitted defaults to "projects"

**Goal:** Verify that omitting `target::` causes `.count` to count projects (the default target).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .count`
**Expected Output:** stdout contains a non-negative integer equal to the project count (same as EC-1 with `target::projects`).
**Verification:**
- Exit code is 0
- stdout contains a numeric value
- Output matches the result of `clg .count target::projects`
**Pass Criteria:** exit 0 + numeric output matching project count (default applied)
**Source:** [params.md](../../params.md)

### EC-7: target::sessions without project:: counts all sessions

**Goal:** Verify that `target::sessions` without `project::` counts sessions across all projects (not just the current project).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .count target::sessions`
**Expected Output:** stdout contains a count that reflects sessions across all projects in storage (not limited to the current directory project).
**Verification:**
- Exit code is 0
- stdout contains a numeric value ≥ number of sessions in a single project
- Count is consistent with total sessions across the fixture
**Pass Criteria:** exit 0 + count reflects all sessions in storage (no implicit project filter)
**Source:** [params.md](../../params.md)

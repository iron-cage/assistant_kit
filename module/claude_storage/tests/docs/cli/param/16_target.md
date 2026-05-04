# Parameter :: `target::`

Edge case tests for the `target::` parameter. Tests validate enum parsing and hierarchy requirements.

**Source:** [params.md#parameter--16-target](../../../../docs/cli/params.md#parameter--16-target) | [types.md#targettype](../../../../docs/cli/types.md#targettype)

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

**Total:** 7 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases

---

### EC-1: Value "projects" accepted

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .count target::projects`
- **Then:** stdout contains a non-negative integer representing the number of projects.; numeric output representing project count
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-2: Value "sessions" accepted

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .count target::sessions`
- **Then:** stdout contains a non-negative integer representing the number of sessions.; numeric output representing session count
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-3: Value "entries" accepted

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .count target::entries session_id::-default_topic`
- **Then:** stdout contains a non-negative integer representing the number of entries.; numeric output representing entry count
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-4: Value "SESSIONS" accepted (case-insensitive)

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .count target::SESSIONS`
- **Then:** No error; output is identical to using lowercase `target::sessions`.; numeric output (case normalization applied)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-5: Invalid value "files" rejected with error

- **Given:** clean environment
- **When:** `clg .count target::files`
- **Then:** stderr contains `target must be projects|sessions|entries, got files`; error message `target must be projects|sessions|entries, got files`
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-6: Omitted defaults to "projects"

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .count`
- **Then:** stdout contains a non-negative integer equal to the project count (same as EC-1 with `target::projects`).; numeric output matching project count (default applied)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-7: target::sessions without project:: counts all sessions

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .count target::sessions`
- **Then:** stdout contains a count that reflects sessions across all projects in storage (not limited to the current directory project).; count reflects all sessions in storage (no implicit project filter)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

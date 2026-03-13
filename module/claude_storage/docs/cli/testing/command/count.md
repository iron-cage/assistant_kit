# Command :: `.count`

Integration tests for the `.count` command. Tests verify counting at each granularity level and scope requirements.

**Source:** [commands.md#command--4-count](../../commands.md#command--4-count)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Default count returns project count | Read Operations |
| IT-2 | target::sessions with project:: returns session count | Read Operations |
| IT-3 | target::entries with project:: and session:: returns entry count | Read Operations |
| IT-4 | Output is a single integer line | Output Format |
| IT-5 | Exit code 0 on success | Exit Codes |
| IT-6 | Exit code 1 on invalid target value | Exit Codes |
| IT-7 | target::sessions with no project:: counts all sessions | Read Operations |
| IT-8 | target::entries with no session:: counts all entries in project | Read Operations |

## Test Coverage Summary

- Read Operations: 5 tests (IT-1, IT-2, IT-3, IT-7, IT-8)
- Output Format: 1 test (IT-4)
- Exit Codes: 2 tests (IT-5, IT-6)

## Test Cases

### IT-1: Default count returns project count

**Goal:** Verify `.count` with no arguments outputs the total number of projects in storage.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 3 projects)
**Command:** `clg .count`
**Expected Output:** A single line containing the integer `3`.
**Verification:**
- stdout is a single line containing exactly `3`
- the value matches the fixture project count
- stderr is empty
**Pass Criteria:** exit 0 + output is `3`

**Source:** [commands.md](../../commands.md)

---

### IT-2: target::sessions with project:: returns session count

**Goal:** Verify `target::sessions project::PROJECT` outputs the session count for the named project.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha` with 4 sessions)
**Command:** `clg .count target::sessions project::alpha`
**Expected Output:** A single line containing the integer `4`.
**Verification:**
- stdout is a single line containing exactly `4`
- the value matches the number of sessions in project `alpha`
- sessions from other projects are not counted
- stderr is empty
**Pass Criteria:** exit 0 + output is `4`

**Source:** [commands.md](../../commands.md)

---

### IT-3: target::entries with project:: and session:: returns entry count

**Goal:** Verify `target::entries project::PROJECT session::SESSION` outputs the entry count for the specified session.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha`, session `s1` with 7 entries)
**Command:** `clg .count target::entries project::alpha session::s1`
**Expected Output:** A single line containing the integer `7`.
**Verification:**
- stdout is a single line containing exactly `7`
- the value matches the number of entries in session `s1` of project `alpha`
- entries from other sessions are not counted
- stderr is empty
**Pass Criteria:** exit 0 + output is `7`

**Source:** [commands.md](../../commands.md)

---

### IT-4: Output is a single integer line

**Goal:** Verify `.count` output is always a bare integer on a single line with no extra formatting, labels, or whitespace.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 2 projects)
**Command:** `clg .count`
**Expected Output:** Exactly `2\n` — a single integer followed by a newline, nothing else.
**Verification:**
- stdout trimmed equals `2`
- no label text (e.g., no `projects:` prefix)
- no table formatting or extra lines
- output can be consumed directly by shell arithmetic (`N=$(clg .count)`)
**Pass Criteria:** exit 0 + stdout is exactly the bare integer

**Source:** [commands.md](../../commands.md)

---

### IT-5: Exit code 0 on success

**Goal:** Verify `.count` exits with code `0` when the count operation succeeds.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (any valid fixture)
**Command:** `clg .count`
**Expected Output:** A single integer on stdout.
**Verification:**
- `$?` is `0` after command completes
- stdout is non-empty and contains an integer
- stderr is empty
**Pass Criteria:** exit 0

**Source:** [commands.md](../../commands.md)

---

### IT-6: Exit code 1 on invalid target value

**Goal:** Verify `.count` exits with code `1` when `target::` is set to an unrecognized value.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .count target::widgets`
**Expected Output:** Error message on stderr indicating invalid target; no count output on stdout.
**Verification:**
- `$?` is `1`
- stderr contains an error message referencing the invalid value `widgets`
- stdout is empty
**Pass Criteria:** exit 1 + error message on stderr for unknown `target::widgets`

**Source:** [commands.md](../../commands.md)

---

### IT-7: target::sessions with no project:: counts all sessions

**Goal:** Verify `target::sessions` without `project::` counts sessions across all projects in storage.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 3 projects with 2, 3, and 1 sessions respectively; total 6 sessions)
**Command:** `clg .count target::sessions`
**Expected Output:** A single line containing `6` (total sessions across all projects).
**Verification:**
- stdout is a single line containing exactly `6`
- the value is the sum of all sessions across all projects in the fixture
- stderr is empty
**Pass Criteria:** exit 0 + output is `6` (global session count)

**Source:** [commands.md](../../commands.md)

---

### IT-8: target::entries with no session:: counts all entries in project

**Goal:** Verify `target::entries project::PROJECT` without `session::` counts all entries across all sessions in the project.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha` with 2 sessions: `s1` has 5 entries, `s2` has 3 entries; total 8 entries in `alpha`)
**Command:** `clg .count target::entries project::alpha`
**Expected Output:** A single line containing `8` (total entries across all sessions in project `alpha`).
**Verification:**
- stdout is a single line containing exactly `8`
- the value is the sum of entries from all sessions in project `alpha`
- entries from other projects are not included
- stderr is empty
**Pass Criteria:** exit 0 + output is `8` (total entries in project `alpha`)

**Source:** [commands.md](../../commands.md)

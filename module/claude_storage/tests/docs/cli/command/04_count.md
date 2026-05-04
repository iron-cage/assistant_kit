# Command :: `.count`

Integration tests for the `.count` command. Tests verify counting at each granularity level and scope requirements.

**Source:** [commands.md#command--4-count](../../../../docs/cli/commands.md#command--4-count)

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

---

### IT-1: Default count returns project count

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 3 projects)
- **When:** `clg .count`
- **Then:** A single line containing the integer `3`.; output is `3`
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-2: target::sessions with project:: returns session count

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha` with 4 sessions)
- **When:** `clg .count target::sessions project::alpha`
- **Then:** A single line containing the integer `4`.; + output is `4`
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-3: target::entries with project:: and session:: returns entry count

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha`, session `s1` with 7 entries)
- **When:** `clg .count target::entries project::alpha session::s1`
- **Then:** A single line containing the integer `7`.; + output is `7`
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-4: Output is a single integer line

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 2 projects)
- **When:** `clg .count`
- **Then:** Exactly `2\n` — a single integer followed by a newline, nothing else.; + stdout is exactly the bare integer
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-5: Exit code 0 on success

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (any valid fixture)
- **When:** `clg .count`
- **Then:** A single integer on stdout.
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-6: Exit code 1 on invalid target value

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .count target::widgets`
- **Then:** Error message on stderr indicating invalid target; no count output on stdout.; + error message on stderr for unknown `target::widgets`
- **Exit:** 1
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-7: target::sessions with no project:: counts all sessions

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 3 projects with 2, 3, and 1 sessions respectively; total 6 sessions)
- **When:** `clg .count target::sessions`
- **Then:** A single line containing `6` (total sessions across all projects).; + output is `6` (global session count)
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-8: target::entries with no session:: counts all entries in project

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha` with 2 sessions: `s1` has 5 entries, `s2` has 3 entries; total 8 entries in `alpha`)
- **When:** `clg .count target::entries project::alpha`
- **Then:** A single line containing `8` (total entries across all sessions in project `alpha`).; + output is `8` (total entries in project `alpha`)
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

# Command :: `.count`

Integration tests for the `.count` command. Tests verify counting at each granularity level and scope requirements.

**Source:** [command/04_count.md](../../../../docs/cli/command/04_count.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| INT-1 | Default count returns project count | Read Operations |
| INT-2 | target::sessions with project:: returns session count | Read Operations |
| INT-3 | target::entries with project:: and session:: returns entry count | Read Operations |
| INT-4 | Output is a single integer line | Output Format |
| INT-5 | Exit code 0 on success | Exit Codes |
| INT-6 | Exit code 1 on invalid target value | Exit Codes |
| INT-7 | target::sessions with no project:: counts all sessions | Read Operations |
| INT-8 | target::entries with no session:: counts all entries in project | Read Operations |

## Test Coverage Summary

- Read Operations: 5 tests (INT-1, INT-2, INT-3, INT-7, INT-8)
- Output Format: 1 test (INT-4)
- Exit Codes: 2 tests (INT-5, INT-6)

## Test Cases

---

### INT-1: Default count returns project count

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .count
```

**Expected behavior:**
- Fixture: 3 projects
- A single line containing the integer `3`
- Exit code: 0
- **Source:** [command/04_count.md](../../../../docs/cli/command/04_count.md)

---

### INT-2: target::sessions with project:: returns session count

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .count target::sessions project::alpha
```

**Expected behavior:**
- Fixture: project `alpha` with 4 sessions
- A single line containing the integer `4`
- Exit code: 0
- **Source:** [command/04_count.md](../../../../docs/cli/command/04_count.md)

---

### INT-3: target::entries with project:: and session:: returns entry count

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .count target::entries project::alpha session::s1
```

**Expected behavior:**
- Fixture: project `alpha`, session `s1` with 7 entries
- A single line containing the integer `7`
- Exit code: 0
- **Source:** [command/04_count.md](../../../../docs/cli/command/04_count.md)

---

### INT-4: Output is a single integer line

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .count
```

**Expected behavior:**
- Fixture: 2 projects
- Exactly `2\n` — a single integer followed by a newline, nothing else
- Exit code: 0
- **Source:** [command/04_count.md](../../../../docs/cli/command/04_count.md)

---

### INT-5: Exit code 0 on success

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .count
```

**Expected behavior:**
- Fixture: any valid fixture
- A single integer on stdout
- Exit code: 0
- **Source:** [command/04_count.md](../../../../docs/cli/command/04_count.md)

---

### INT-6: Exit code 1 on invalid target value

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .count target::widgets
```

**Expected behavior:**
- Error message on stderr indicating invalid target; no count output on stdout
- Exit code: 1
- **Source:** [command/04_count.md](../../../../docs/cli/command/04_count.md)

---

### INT-7: target::sessions with no project:: counts all sessions

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .count target::sessions
```

**Expected behavior:**
- Fixture: 3 projects with 2, 3, and 1 sessions respectively; total 6 sessions
- A single line containing `6` (total sessions across all projects)
- Exit code: 0
- **Source:** [command/04_count.md](../../../../docs/cli/command/04_count.md)

---

### INT-8: target::entries with no session:: counts all entries in project

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .count target::entries project::alpha
```

**Expected behavior:**
- Fixture: project `alpha` with 2 sessions: `s1` has 5 entries, `s2` has 3 entries; total 8 entries in `alpha`
- A single line containing `8` (total entries across all sessions in project `alpha`)
- Exit code: 0
- **Source:** [command/04_count.md](../../../../docs/cli/command/04_count.md)

# Command :: `.status`

Integration tests for the `.status` command. Tests verify storage overview, statistics output, and verbosity behavior.

**Source:** [command/01_status.md](../../../../docs/cli/command/01_status.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| INT-1 | Default output with real storage | Read Operations |
| INT-2 | Verbosity 0 machine-readable output | Output Format |
| INT-3 | Verbosity 2 detailed per-project output | Output Format |
| INT-4 | Custom storage path via path:: | Configuration |
| INT-5 | Custom storage path via CLAUDE_STORAGE_ROOT env | Configuration |
| INT-6 | Exit code 0 on success | Exit Codes |
| INT-7 | Exit code 2 on unreadable storage path | Exit Codes |
| INT-8 | Output contains project count and session count | Read Operations |

## Test Coverage Summary

- Read Operations: 2 tests (INT-1, INT-8)
- Output Format: 2 tests (INT-2, INT-3)
- Configuration: 2 tests (INT-4, INT-5)
- Exit Codes: 2 tests (INT-6, INT-7)

## Test Cases

---

### INT-1: Default output with real storage

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .status
```

**Expected behavior:**
- Fixture: 2 projects, 3 sessions total in storage
- Stdout contains a summary table with project count `2` and session count `3`
- Exit code: 0
- **Source:** [command/01_status.md](../../../../docs/cli/command/01_status.md)

---

### INT-2: Verbosity 0 machine-readable output

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .status verbosity::0
```

**Expected behavior:**
- Fixture: 2 projects, 3 sessions total in storage
- Stdout contains exactly `projects: 2` and `sessions: 3` (no table borders, no labels)
- Exit code: 0
- **Source:** [command/01_status.md](../../../../docs/cli/command/01_status.md)

---

### INT-3: Verbosity 2 detailed per-project output

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .status verbosity::2
```

**Expected behavior:**
- Fixture: 2 projects, each with 1 session containing known entry counts
- Stdout contains summary table plus per-project rows showing session counts and entry type breakdown (user/assistant)
- Exit code: 0
- **Source:** [command/01_status.md](../../../../docs/cli/command/01_status.md)

---

### INT-4: Custom storage path via path::

**Command:**
```
clg .status path::/tmp/alt-fixture
```

**Expected behavior:**
- Fixture: `/tmp/alt-fixture` has 1 project, 1 session; default storage has different counts
- Stdout shows counts from `/tmp/alt-fixture` (1 project, 1 session), not from default storage
- Exit code: 0
- **Source:** [command/01_status.md](../../../../docs/cli/command/01_status.md)

---

### INT-5: Custom storage path via CLAUDE_STORAGE_ROOT env

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .status
```

**Expected behavior:**
- Fixture: 2 projects, 3 sessions in `CLAUDE_STORAGE_ROOT`
- Stdout reflects the fixture's 2 projects and 3 sessions, not the real `~/.claude/` counts
- Exit code: 0
- **Source:** [command/01_status.md](../../../../docs/cli/command/01_status.md)

---

### INT-6: Exit code 0 on success

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .status
```

**Expected behavior:**
- Normal summary output on stdout
- Exit code: 0
- **Source:** [command/01_status.md](../../../../docs/cli/command/01_status.md)

---

### INT-7: Exit code 2 on unreadable storage path

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/nonexistent-storage-xyz clg .status
```

**Expected behavior:**
- Error message on stderr indicating storage read failure
- No summary on stdout
- Exit code: 2
- **Source:** [command/01_status.md](../../../../docs/cli/command/01_status.md)

---

### INT-8: Output contains project count and session count

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .status
```

**Expected behavior:**
- Fixture: 3 projects, 5 sessions total in storage
- Output includes labeled project count (`3`) and labeled session count (`5`)
- Exit code: 0
- **Source:** [command/01_status.md](../../../../docs/cli/command/01_status.md)

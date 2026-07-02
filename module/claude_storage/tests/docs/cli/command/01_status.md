# Command :: `.status`

Integration tests for the `.status` command. Tests verify storage overview, statistics output, and show_tokens behavior.

**Source:** [command/01_status.md](../../../../docs/cli/command/01_status.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| INT-1 | Default output with real storage | Read Operations |
| INT-2 | show_tokens::1 includes token usage section | Output Control |
| INT-3 | show_tokens::0 default omits token usage (fast path) | Output Control |
| INT-4 | Custom storage path via path:: | Configuration |
| INT-5 | Custom storage path via CLAUDE_STORAGE_ROOT env | Configuration |
| INT-6 | Exit code 0 on success | Exit Codes |
| INT-7 | Exit code 2 on unreadable storage path | Exit Codes |
| INT-8 | Output contains project count and session count | Read Operations |
| INT-9 | show_tokens:: with invalid value rejected | Invalid Parameter Rejection |

## Test Coverage Summary

- Read Operations: 2 tests (INT-1, INT-8)
- Output Control: 2 tests (INT-2, INT-3)
- Configuration: 2 tests (INT-4, INT-5)
- Exit Codes: 2 tests (INT-6, INT-7)
- Invalid Parameter Rejection: 1 test (INT-9)

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

### INT-2: show_tokens::1 includes token usage section

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .status show_tokens::1
```

**Expected behavior:**
- Fixture: at least 1 project with 1 session containing entries with token data
- Stdout includes standard project/session counts
- Stdout includes Tokens section with Input, Output, Cache Read, Cache Creation
- Exit code: 0
- **Source:** [command/01_status.md](../../../../docs/cli/command/01_status.md)

---

### INT-3: show_tokens::0 default omits token usage (fast path)

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .status
```

**Expected behavior:**
- Fixture: storage with entries containing token data
- Stdout contains project and session counts but NO Tokens section
- Command completes quickly (filesystem-only, no JSONL parsing)
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

---

### INT-9: show_tokens:: with invalid value rejected

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .status show_tokens::invalid
```

**Expected behavior:**
- `show_tokens::invalid` is not a valid boolean value (accepted: `0`, `1`)
- Error message on stderr describing the argument error
- No storage output on stdout
- Exit code: 1
- **Source:** [command/01_status.md](../../../../docs/cli/command/01_status.md)

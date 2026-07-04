# Command :: `.tail`

Integration tests for the `.tail` command. Tests verify zero-parameter defaults, entry count control, topic resolution, and not-found handling.

**Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| INT-1 | No args prints last 4 entries of default_topic session | Location-Aware |
| INT-2 | tail::N controls entry count | Entry Count |
| INT-3 | tail::0 prints all entries | Entry Count |
| INT-4 | topic:: resolves a non-default session | Topic Resolution |
| INT-5 | path:: resolves a different directory's project | Project Scope |
| INT-6 | Fewer entries than requested prints all available | Boundary |
| INT-7 | Exit code 2 when cwd has no project | Exit Codes |

## Test Coverage Summary

- Location-Aware: 1 test (INT-1)
- Entry Count: 2 tests (INT-2, INT-3)
- Topic Resolution: 1 test (INT-4)
- Project Scope: 1 test (INT-5)
- Boundary: 1 test (INT-6)
- Exit Codes: 1 test (INT-7)

## Test Cases

---

### INT-1: No args prints last 4 entries of default_topic session

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail
```

**Expected behavior:**
- Fixture: a project whose path-encoding matches the test's cwd, with a `-default_topic` session containing 6 known entries
- The last 4 entries printed, oldest-first, as conversation content
- Exit code: 0
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-2: tail::N controls entry count

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail tail::2
```

**Expected behavior:**
- Fixture: same project, `-default_topic` session with 6 known entries
- Exactly the last 2 entries printed, oldest-first
- Exit code: 0
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-3: tail::0 prints all entries

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail tail::0
```

**Expected behavior:**
- Fixture: `-default_topic` session with 6 known entries
- All 6 entries printed, oldest-first
- Exit code: 0
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-4: topic:: resolves a non-default session

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail topic::work
```

**Expected behavior:**
- Fixture: project with both a `-default_topic` session and a `-work` session, each with distinct known content
- The last 4 entries from the `-work` session printed; no `-default_topic` content shown
- Exit code: 0
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-5: path:: resolves a different directory's project

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail path::/home/alice/projects/alpha
```

**Expected behavior:**
- Fixture: project `alpha` with a `-default_topic` session; run from a cwd that does not correspond to `alpha`
- The last 4 entries from `alpha`'s `-default_topic` session printed
- Exit code: 0
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-6: Fewer entries than requested prints all available

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail tail::10
```

**Expected behavior:**
- Fixture: `-default_topic` session with only 3 known entries
- All 3 entries printed, oldest-first; no error or padding
- Exit code: 0
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-7: Exit code 2 when cwd has no project

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail
```

**Expected behavior:**
- Fixture: run from a directory (e.g., `/tmp`) that has no matching storage project
- Error message on stderr indicating the current directory has no project in storage
- Exit code: 2
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

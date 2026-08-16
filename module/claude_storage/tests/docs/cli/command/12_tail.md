# Command :: `.tail`

Integration tests for the `.tail` command. Tests verify zero-parameter defaults, entry count control, topic resolution, the most-recently-modified-session fallback, and not-found handling.

**Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| INT-1 | No args, single session present, prints its last 4 entries | Location-Aware |
| INT-2 | tail::N controls entry count | Entry Count |
| INT-3 | tail::0 prints all entries | Entry Count |
| INT-4 | topic:: resolves a non-default session | Topic Resolution |
| INT-5 | path:: resolves a different directory's project | Project Scope |
| INT-6 | Fewer entries than requested prints all available | Boundary |
| INT-7 | Exit code 2 when cwd has no project | Exit Codes |
| INT-8 | Negative tail:: is rejected with exit code 1 | Input Validation |
| INT-9 | No args falls back to the most recent session when no `-default_topic` session exists | Recency Fallback |
| INT-10 | No args picks the most recently modified session among multiple candidates | Recency Fallback |
| INT-11 | No args excludes agent sessions from the most-recent fallback | Recency Fallback |

## Test Coverage Summary

- Location-Aware: 1 test (INT-1)
- Entry Count: 2 tests (INT-2, INT-3)
- Topic Resolution: 1 test (INT-4)
- Project Scope: 1 test (INT-5)
- Boundary: 1 test (INT-6)
- Exit Codes: 1 test (INT-7)
- Input Validation: 1 test (INT-8)
- Recency Fallback: 3 tests (INT-9, INT-10, INT-11)

## Test Cases

---

### INT-1: No args, single session present, prints its last 4 entries

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail
```

**Expected behavior:**
- Fixture: a project whose path-encoding matches the test's cwd, with a `-default_topic` session containing 6 known entries
- The last 4 entries printed, oldest-first, as conversation content
- Exit code: 0
- The session is selected by the most-recently-modified-non-agent-session fallback (BUG-488); it is the only session in the fixture, so it is trivially both the default-named and the most recent one — this test does not by itself distinguish name-based resolution from recency-based resolution (see INT-9 for that distinction)
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

---

### INT-8: Negative tail:: is rejected with exit code 1

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail tail::-1
```

**Expected behavior:**
- Fixture: same project, `-default_topic` session with 6 known entries (rejection happens before entries are loaded)
- Error message on stderr: exactly `"tail must be non-negative"`
- Exit code: 1
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md), [param/25_tail.md](../../../../docs/cli/param/25_tail.md)

---

### INT-9: No args falls back to the most recent session when no `-default_topic` session exists

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail
```

**Expected behavior:**
- Fixture: a project with exactly one UUID-named session (no `-default_topic` session written at all), containing 6 known entries
- The last 4 entries printed, oldest-first, from the UUID-named session
- Exit code: 0
- Regression coverage for BUG-488: real Claude Code sessions are UUID-named, never topic-tagged, so the zero-parameter default must not require a literal `-default_topic` session to exist
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-10: No args picks the most recently modified session among multiple candidates

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail
```

**Expected behavior:**
- Fixture: a project with two UUID-named sessions written with distinguishable mtimes (older written first, then a delay, then the newer)
- Output contains the newer session's marker content; the older session's content is not required to resolve the assertion
- Exit code: 0
- Confirms the fallback actually compares modification times rather than picking an arbitrary/first-found session
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

---

### INT-11: No args excludes agent sessions from the most-recent fallback

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .tail
```

**Expected behavior:**
- Fixture: a project with one main (UUID-named) session and one `agent-*`-named sidecar session, the agent session written more recently (newer mtime) than the main session
- Output contains the main session's content; the agent session's content is absent
- Exit code: 0
- Confirms agent sidecar sessions are never selected by the recency fallback even when they are the newest file in the project, matching `claude_storage_core::continuation`'s own established agent-exclusion convention
- **Source:** [command/12_tail.md](../../../../docs/cli/command/12_tail.md)

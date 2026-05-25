# Command :: `.show`

Integration tests for the `.show` command. Tests verify project view, session view, location-aware behavior, and display modes.

**Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| INT-1 | No args shows current project's sessions | Location-Aware |
| INT-2 | session_id:: shows conversation content | Session View |
| INT-3 | project:: selects explicit project | Project View |
| INT-4 | session_id:: + project:: shows session in named project | Combined |
| INT-5 | metadata::1 suppresses content, shows metadata | Display Mode |
| INT-6 | entries::1 shows all session entries | Display Mode |
| INT-7 | Exit code 2 when cwd has no project | Exit Codes |
| INT-8 | project:: with path-encoded ID | Project View |

## Test Coverage Summary

- Location-Aware: 1 test (INT-1)
- Session View: 1 test (INT-2)
- Project View: 2 tests (INT-3, INT-8)
- Combined: 1 test (INT-4)
- Display Mode: 2 tests (INT-5, INT-6)
- Exit Codes: 1 test (INT-7)

## Test Cases

---

### INT-1: No args shows current project's sessions

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show
```

**Expected behavior:**
- Fixture: a project whose path-encoding matches the test's cwd; run from that cwd
- A session list for the project corresponding to cwd; each session shown with its ID and basic metadata
- Exit code: 0
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md)

---

### INT-2: session_id:: shows conversation content

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show session_id::-default_topic
```

**Expected behavior:**
- Fixture: project `alpha` with session `-default_topic` containing known messages
- Session summary or content for session `-default_topic`; includes session ID in output
- Exit code: 0
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md)

---

### INT-3: project:: selects explicit project

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show project::alpha
```

**Expected behavior:**
- Fixture: projects `alpha` and `beta`; run from a cwd that does not correspond to either project
- Session list for project `alpha`; no sessions from `beta` or any cwd-resolved project
- Exit code: 0
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md)

---

### INT-4: session_id:: + project:: shows session in named project

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show session_id::s1 project::alpha
```

**Expected behavior:**
- Fixture: project `alpha` with session `s1`; project `beta` with a different session `s1`
- Content or summary for session `s1` from project `alpha` specifically, not `s1` from `beta`
- Exit code: 0
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md)

---

### INT-5: metadata::1 suppresses content, shows metadata only

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show session_id::-default_topic metadata::1
```

**Expected behavior:**
- Fixture: session `-default_topic` with known user/assistant messages
- Metadata fields (e.g., entry count, session type, timestamps) present; actual message text absent
- Exit code: 0
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md)

---

### INT-6: entries::1 shows all session entries

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show session_id::-default_topic entries::1
```

**Expected behavior:**
- Fixture: session `-default_topic` with 4 known entries: 2 user, 2 assistant
- All 4 entries from the session shown, including user and assistant message content
- Exit code: 0
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md)

---

### INT-7: Exit code 2 when cwd has no project

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show
```

**Expected behavior:**
- Fixture: run from a directory (e.g., `/tmp`) that has no matching storage project
- Error message on stderr indicating the current directory has no project in storage
- Exit code: 2
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md)

---

### INT-8: project:: with path-encoded ID

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .show project::-home-alice-projects-alpha
```

**Expected behavior:**
- Fixture: project stored with path-encoded ID `-home-alice-projects-alpha`
- Session list for the project with path-encoded ID `-home-alice-projects-alpha`
- Exit code: 0
- **Source:** [command/03_show.md](../../../../docs/cli/command/03_show.md)

# Command :: `.show`

Integration tests for the `.show` command. Tests verify project view, session view, location-aware behavior, and display modes.

**Source:** [commands.md#command--3-show](../../../../docs/cli/commands.md#command--3-show)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | No args shows current project's sessions | Location-Aware |
| IT-2 | session_id:: shows conversation content | Session View |
| IT-3 | project:: selects explicit project | Project View |
| IT-4 | session_id:: + project:: shows session in named project | Combined |
| IT-5 | metadata::1 suppresses content, shows metadata | Display Mode |
| IT-6 | entries::1 shows all session entries | Display Mode |
| IT-7 | Exit code 2 when cwd has no project | Exit Codes |
| IT-8 | project:: with path-encoded ID | Project View |

## Test Coverage Summary

- Location-Aware: 1 test (IT-1)
- Session View: 1 test (IT-2)
- Project View: 2 tests (IT-3, IT-8)
- Combined: 1 test (IT-4)
- Display Mode: 2 tests (IT-5, IT-6)
- Exit Codes: 1 test (IT-7)

## Test Cases

---

### IT-1: No args shows current project's sessions

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture includes a project whose path-encoding matches the test's cwd); run from that cwd.
- **When:** `clg .show`
- **Then:** A session list for the project corresponding to cwd; each session shown with its ID and basic metadata.; session list for the cwd-resolved project
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-2: session_id:: shows conversation content

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha` with session `-default_topic` containing known messages)
- **When:** `clg .show session_id::-default_topic`
- **Then:** Session summary or content for session `-default_topic`; includes session ID in output.; + session content/summary for `-default_topic` displayed
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-3: project:: selects explicit project

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: projects `alpha` and `beta`; run from a cwd that does not correspond to either project)
- **When:** `clg .show project::alpha`
- **Then:** Session list for project `alpha`; no sessions from `beta` or any cwd-resolved project.; + session list scoped to project `alpha`
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-4: session_id:: + project:: shows session in named project

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha` with session `s1`; project `beta` with a different session `s1`)
- **When:** `clg .show session_id::s1 project::alpha`
- **Then:** Content or summary for session `s1` from project `alpha` specifically, not `s1` from `beta`.; + session `s1` from project `alpha` shown
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-5: metadata::1 suppresses content, shows metadata only

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session `-default_topic` with known user/assistant messages)
- **When:** `clg .show session_id::-default_topic metadata::1`
- **Then:** Metadata fields (e.g., entry count, session type, timestamps) present; actual message text absent.; + metadata fields present, message content absent
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-6: entries::1 shows all session entries

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session `-default_topic` with 4 known entries: 2 user, 2 assistant)
- **When:** `clg .show session_id::-default_topic entries::1`
- **Then:** All 4 entries from the session shown, including user and assistant message content.; + all session entries present in output
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-7: Exit code 2 when cwd has no project

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`; run from a directory (e.g., `/tmp`) that has no matching storage project.
- **When:** `clg .show`
- **Then:** Error message on stderr indicating the current directory has no project in storage.; + error message on stderr
- **Exit:** 2
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-8: project:: with path-encoded ID

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project stored with path-encoded ID `-home-user1-pro-alpha`)
- **When:** `clg .show project::-home-user1-pro-alpha`
- **Then:** Session list for the project with path-encoded ID `-home-user1-pro-alpha`.; + sessions for path-encoded project shown
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

# Command :: `.show`

Integration tests for the `.show` command. Tests verify project view, session view, location-aware behavior, and display modes.

**Source:** [commands.md#command--3-show](../../../../../docs/cli/commands.md#command--3-show)

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

### IT-1: No args shows current project's sessions

**Goal:** Verify `.show` with no arguments resolves cwd to its storage project and lists that project's sessions.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture includes a project whose path-encoding matches the test's cwd); run from that cwd.
**Command:** `clg .show`
**Expected Output:** A session list for the project corresponding to cwd; each session shown with its ID and basic metadata.
**Verification:**
- stdout contains session entries belonging to the cwd-resolved project
- no session entries from other projects appear
- stdout does not contain raw JSONL content (entries not expanded by default)
- stderr is empty
**Pass Criteria:** exit 0 + session list for the cwd-resolved project

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-2: session_id:: shows conversation content

**Goal:** Verify `session_id::` parameter causes the command to display the conversation content of the specified session.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha` with session `-default_topic` containing known messages)
**Command:** `clg .show session_id::-default_topic`
**Expected Output:** Session summary or content for session `-default_topic`; includes session ID in output.
**Verification:**
- stdout contains the session ID `-default_topic`
- stdout contains session-level information (entry count, timestamps, or content preview)
- stdout is different from project-level listing output
- stderr is empty
**Pass Criteria:** exit 0 + session content/summary for `-default_topic` displayed

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-3: project:: selects explicit project

**Goal:** Verify `project::` parameter selects a named project and shows its session list regardless of cwd.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: projects `alpha` and `beta`; run from a cwd that does not correspond to either project)
**Command:** `clg .show project::alpha`
**Expected Output:** Session list for project `alpha`; no sessions from `beta` or any cwd-resolved project.
**Verification:**
- stdout contains session entries for project `alpha`
- stdout does not contain session entries for project `beta`
- cwd is not used for project resolution (explicit `project::alpha` takes precedence)
- stderr is empty
**Pass Criteria:** exit 0 + session list scoped to project `alpha`

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-4: session_id:: + project:: shows session in named project

**Goal:** Verify combining `session_id::` and `project::` shows the specified session from the specified project.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha` with session `s1`; project `beta` with a different session `s1`)
**Command:** `clg .show session_id::s1 project::alpha`
**Expected Output:** Content or summary for session `s1` from project `alpha` specifically, not `s1` from `beta`.
**Verification:**
- stdout contains session `s1` content scoped to project `alpha`
- content is consistent with the `alpha/s1` fixture data, not `beta/s1`
- stderr is empty
**Pass Criteria:** exit 0 + session `s1` from project `alpha` shown

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-5: metadata::1 suppresses content, shows metadata only

**Goal:** Verify `metadata::1` outputs only session metadata (timestamps, entry counts, type) without showing message content.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session `-default_topic` with known user/assistant messages)
**Command:** `clg .show session_id::-default_topic metadata::1`
**Expected Output:** Metadata fields (e.g., entry count, session type, timestamps) present; actual message text absent.
**Verification:**
- stdout contains metadata fields (entry count or timestamp or session type indicator)
- stdout does not contain the actual user/assistant message text from the fixture
- output is shorter than `entries::1` output for the same session
- stderr is empty
**Pass Criteria:** exit 0 + metadata fields present, message content absent

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-6: entries::1 shows all session entries

**Goal:** Verify `entries::1` expands the session view to include all individual conversation entries.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session `-default_topic` with 4 known entries: 2 user, 2 assistant)
**Command:** `clg .show session_id::-default_topic entries::1`
**Expected Output:** All 4 entries from the session shown, including user and assistant message content.
**Verification:**
- stdout contains all 4 entry outputs
- both user and assistant message content from the fixture appears
- output is longer/more detailed than `clg .show session_id::-default_topic` without `entries::1`
- stderr is empty
**Pass Criteria:** exit 0 + all session entries present in output

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-7: Exit code 2 when cwd has no project

**Goal:** Verify `.show` without `project::` exits with code `2` when cwd does not correspond to any project in storage.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`; run from a directory (e.g., `/tmp`) that has no matching storage project.
**Command:** `clg .show`
**Expected Output:** Error message on stderr indicating the current directory has no project in storage.
**Verification:**
- `$?` is `2`
- stderr contains an error message (e.g., "no project found" or "not found in storage")
- stdout is empty
**Pass Criteria:** exit 2 + error message on stderr

**Source:** [commands.md](../../../../../docs/cli/commands.md)

---

### IT-8: project:: with path-encoded ID

**Goal:** Verify `project::` accepts a path-encoded project identifier (hyphen-separated path segments) and resolves correctly.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project stored with path-encoded ID `-home-user1-pro-alpha`)
**Command:** `clg .show project::-home-user1-pro-alpha`
**Expected Output:** Session list for the project with path-encoded ID `-home-user1-pro-alpha`.
**Verification:**
- stdout contains session entries for the path-encoded project
- stdout does not display an error about invalid project ID
- output matches what `clg .show project::alpha` would show for the same project (if alias exists)
- stderr is empty
**Pass Criteria:** exit 0 + sessions for path-encoded project shown

**Source:** [commands.md](../../../../../docs/cli/commands.md)

# Command :: `.list`

Integration tests for the `.list` command. Tests verify project listing, session display, filtering, and smart auto-enable behavior.

**Source:** [commands.md#command--2-list](../../commands.md#command--2-list)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Default list shows all projects | Read Operations |
| IT-2 | type::uuid filters to UUID projects only | Filtering |
| IT-3 | type::path filters to path-encoded projects only | Filtering |
| IT-4 | sessions::1 expands session list per project | Session Display |
| IT-5 | path:: substring filters project list | Filtering |
| IT-6 | session:: auto-enables sessions display | Auto-Enable |
| IT-7 | agent::1 filters to agent sessions only | Filtering |
| IT-8 | agent::0 filters to main sessions only | Filtering |
| IT-9 | min_entries:: auto-enables sessions display | Auto-Enable |
| IT-10 | sessions::0 suppresses display even with session:: | Override |
| IT-11 | Combined path:: session:: filter | Filtering |
| IT-12 | Exit code 0 on empty storage | Exit Codes |

## Test Coverage Summary

- Read Operations: 1 test (IT-1)
- Filtering: 5 tests (IT-2, IT-3, IT-5, IT-7, IT-8)
- Session Display: 1 test (IT-4)
- Auto-Enable: 2 tests (IT-6, IT-9)
- Override: 1 test (IT-10)
- Filtering (combined): 1 test (IT-11)
- Exit Codes: 1 test (IT-12)

## Test Cases

### IT-1: Default list shows all projects

**Goal:** Verify `.list` with no arguments outputs every project in storage, one per line.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 3 projects named `alpha`, `beta`, `gamma`)
**Command:** `clg .list`
**Expected Output:** Three project entries in stdout, each showing a project path or identifier.
**Verification:**
- stdout contains exactly 3 project entries
- each fixture project name/path appears in output (`alpha`, `beta`, `gamma`)
- no session detail lines appear (sessions not expanded by default)
- stderr is empty
**Pass Criteria:** exit 0 + all 3 projects listed, no session expansion

**Source:** [commands.md](../../commands.md)

---

### IT-2: type::uuid filters to UUID projects only

**Goal:** Verify `type::uuid` restricts output to projects whose identifiers are UUIDs, excluding path-encoded projects.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 1 UUID project `a1b2c3d4-...`, 2 path-encoded projects)
**Command:** `clg .list type::uuid`
**Expected Output:** Only the 1 UUID project appears; path-encoded projects are absent.
**Verification:**
- stdout contains exactly 1 project entry
- the entry matches UUID format (`[0-9a-f]{8}-[0-9a-f]{4}-...`)
- path-encoded project identifiers do not appear in stdout
**Pass Criteria:** exit 0 + only UUID-identified projects in output

**Source:** [commands.md](../../commands.md)

---

### IT-3: type::path filters to path-encoded projects only

**Goal:** Verify `type::path` restricts output to path-encoded projects, excluding UUID projects.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 2 path-encoded projects, 1 UUID project)
**Command:** `clg .list type::path`
**Expected Output:** Only the 2 path-encoded projects appear; the UUID project is absent.
**Verification:**
- stdout contains exactly 2 project entries
- neither entry matches UUID format
- path-encoded identifiers (hyphen-separated path segments) appear in output
- UUID project does not appear
**Pass Criteria:** exit 0 + only path-encoded projects in output

**Source:** [commands.md](../../commands.md)

---

### IT-4: sessions::1 expands session list per project

**Goal:** Verify `sessions::1` causes each project row to be followed by its session list.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 2 projects; project `alpha` has 2 sessions, project `beta` has 1 session)
**Command:** `clg .list sessions::1`
**Expected Output:** Both projects listed, each followed by their respective session IDs (3 session entries total nested under their projects).
**Verification:**
- stdout contains both project entries (`alpha`, `beta`)
- stdout contains 3 session ID entries total
- session entries appear under their owning project (ordering/grouping consistent)
- no session entries appear for a project not in the fixture
**Pass Criteria:** exit 0 + per-project session lists expanded with correct counts

**Source:** [commands.md](../../commands.md)

---

### IT-5: path:: substring filters project list

**Goal:** Verify `path::` filters the project list to only those whose path contains the given substring.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: projects at `/home/user1/pro/alpha`, `/home/user1/pro/beta`, `/tmp/other`)
**Command:** `clg .list path::pro`
**Expected Output:** Only projects whose path contains `pro` are listed (`alpha`, `beta`); `/tmp/other` is absent.
**Verification:**
- stdout contains `alpha` project entry
- stdout contains `beta` project entry
- stdout does not contain `/tmp/other` project entry
- exactly 2 projects in output
**Pass Criteria:** exit 0 + only projects matching `pro` substring shown

**Source:** [commands.md](../../commands.md)

---

### IT-6: session:: auto-enables sessions display

**Goal:** Verify that providing `session::` without `sessions::1` automatically expands session lists per project.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 2 projects; session with ID containing `abc` exists in project `alpha`)
**Command:** `clg .list session::abc`
**Expected Output:** Sessions matching `abc` appear in output without requiring explicit `sessions::1`.
**Verification:**
- stdout contains the session entry whose ID contains `abc`
- sessions are shown (auto-enabled) even though `sessions::1` was not passed
- only sessions matching `abc` are shown, not all sessions
- project entry for `alpha` also appears
**Pass Criteria:** exit 0 + matching session visible without explicit `sessions::1`

**Source:** [commands.md](../../commands.md)

---

### IT-7: agent::1 filters to agent sessions only

**Goal:** Verify `agent::1` restricts displayed sessions to agent-type sessions only, excluding main sessions.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha` with 1 agent session and 1 main session)
**Command:** `clg .list agent::1`
**Expected Output:** Only the agent session appears; the main session is not in output.
**Verification:**
- stdout contains the agent session entry
- stdout does not contain the main session entry
- sessions are expanded (auto-enabled by `agent::` parameter)
**Pass Criteria:** exit 0 + only agent sessions shown

**Source:** [commands.md](../../commands.md)

---

### IT-8: agent::0 filters to main sessions only

**Goal:** Verify `agent::0` restricts displayed sessions to main (non-agent) sessions only, excluding agent sessions.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha` with 1 agent session and 1 main session)
**Command:** `clg .list agent::0`
**Expected Output:** Only the main session appears; the agent session is not in output.
**Verification:**
- stdout contains the main session entry
- stdout does not contain the agent session entry
- sessions are expanded (auto-enabled by `agent::` parameter)
**Pass Criteria:** exit 0 + only main (non-agent) sessions shown

**Source:** [commands.md](../../commands.md)

---

### IT-9: min_entries:: auto-enables sessions display

**Goal:** Verify that `min_entries::N` automatically enables session display and filters out sessions below the threshold.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha` has session `s1` with 15 entries and session `s2` with 3 entries)
**Command:** `clg .list min_entries::10`
**Expected Output:** Session `s1` (15 entries) appears; session `s2` (3 entries) does not.
**Verification:**
- stdout contains session `s1` entry
- stdout does not contain session `s2` entry
- sessions are shown (auto-enabled) without explicit `sessions::1`
- project `alpha` entry also appears
**Pass Criteria:** exit 0 + only sessions with >= 10 entries shown, auto-enabled

**Source:** [commands.md](../../commands.md)

---

### IT-10: sessions::0 suppresses display even with session::

**Goal:** Verify that explicit `sessions::0` suppresses session expansion even when `session::` filter is provided.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha` with a session whose ID contains `abc`)
**Command:** `clg .list session::abc sessions::0`
**Expected Output:** Only project entries shown; no session entries appear despite `session::abc` being provided.
**Verification:**
- stdout contains project-level entries
- stdout does not contain any session ID entries
- the `sessions::0` override takes effect over `session::` auto-enable
**Pass Criteria:** exit 0 + no session entries in output

**Source:** [commands.md](../../commands.md)

---

### IT-11: Combined path:: session:: filter

**Goal:** Verify that `path::` and `session::` can be combined to narrow results to sessions within path-matching projects.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha` at path containing `pro` with session `s-abc`; project `other` at unrelated path with session `s-abc`)
**Command:** `clg .list path::pro session::abc`
**Expected Output:** Session `s-abc` under project `alpha` appears; session `s-abc` under project `other` does not (path filter excluded it).
**Verification:**
- stdout contains `alpha` project entry
- stdout contains session `s-abc` under `alpha`
- stdout does not contain `other` project entry
- stdout does not contain session entries from `other`
**Pass Criteria:** exit 0 + only session entries from path-matching projects shown

**Source:** [commands.md](../../commands.md)

---

### IT-12: Exit code 0 on empty storage

**Goal:** Verify `.list` exits with code `0` even when the storage root contains no projects.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/empty-fixture` (fixture: empty `projects/` directory, no project subdirectories)
**Command:** `clg .list`
**Expected Output:** Empty output or a "no projects found" message; no error.
**Verification:**
- `$?` is `0`
- stderr is empty
- stdout is either empty or contains a benign "no projects" indication
**Pass Criteria:** exit 0 + no error for empty storage

**Source:** [commands.md](../../commands.md)

# Command :: `.list`

Integration tests for the `.list` command. Tests verify project listing, session display, filtering, and smart auto-enable behavior.

**Source:** [commands.md#command--2-list](../../../../docs/cli/commands.md#command--2-list)

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

---

### IT-1: Default list shows all projects

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 3 projects named `alpha`, `beta`, `gamma`)
- **When:** `clg .list`
- **Then:** Three project entries in stdout, each showing a project path or identifier.; all 3 projects listed, no session expansion
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-2: type::uuid filters to UUID projects only

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 1 UUID project `a1b2c3d4-...`, 2 path-encoded projects)
- **When:** `clg .list type::uuid`
- **Then:** Only the 1 UUID project appears; path-encoded projects are absent.; + only UUID-identified projects in output
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-3: type::path filters to path-encoded projects only

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 2 path-encoded projects, 1 UUID project)
- **When:** `clg .list type::path`
- **Then:** Only the 2 path-encoded projects appear; the UUID project is absent.; + only path-encoded projects in output
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-4: sessions::1 expands session list per project

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 2 projects; project `alpha` has 2 sessions, project `beta` has 1 session)
- **When:** `clg .list sessions::1`
- **Then:** Both projects listed, each followed by their respective session IDs (3 session entries total nested under their projects).; + per-project session lists expanded with correct counts
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-5: path:: substring filters project list

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: projects at `/home/user1/pro/alpha`, `/home/user1/pro/beta`, `/tmp/other`)
- **When:** `clg .list path::pro`
- **Then:** Only projects whose path contains `pro` are listed (`alpha`, `beta`); `/tmp/other` is absent.; + only projects matching `pro` substring shown
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-6: session:: auto-enables sessions display

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 2 projects; session with ID containing `abc` exists in project `alpha`)
- **When:** `clg .list session::abc`
- **Then:** Sessions matching `abc` appear in output without requiring explicit `sessions::1`.; + matching session visible without explicit `sessions::1`
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-7: agent::1 filters to agent sessions only

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha` with 1 agent session and 1 main session)
- **When:** `clg .list agent::1`
- **Then:** Only the agent session appears; the main session is not in output.; + only agent sessions shown
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-8: agent::0 filters to main sessions only

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha` with 1 agent session and 1 main session)
- **When:** `clg .list agent::0`
- **Then:** Only the main session appears; the agent session is not in output.; + only main (non-agent) sessions shown
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-9: min_entries:: auto-enables sessions display

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha` has session `s1` with 15 entries and session `s2` with 3 entries)
- **When:** `clg .list min_entries::10`
- **Then:** Session `s1` (15 entries) appears; session `s2` (3 entries) does not.; + only sessions with >= 10 entries shown, auto-enabled
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-10: sessions::0 suppresses display even with session::

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha` with a session whose ID contains `abc`)
- **When:** `clg .list session::abc sessions::0`
- **Then:** Only project entries shown; no session entries appear despite `session::abc` being provided.; + no session entries in output
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-11: Combined path:: session:: filter

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha` at path containing `pro` with session `s-abc`; project `other` at unrelated path with session `s-abc`)
- **When:** `clg .list path::pro session::abc`
- **Then:** Session `s-abc` under project `alpha` appears; session `s-abc` under project `other` does not (path filter excluded it).; + only session entries from path-matching projects shown
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-12: Exit code 0 on empty storage

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/empty-fixture` (fixture: empty `projects/` directory, no project subdirectories)
- **When:** `clg .list`
- **Then:** Empty output or a "no projects found" message; no error.; + no error for empty storage
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

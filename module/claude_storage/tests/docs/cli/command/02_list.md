# Command :: `.list`

Integration tests for the `.list` command. Tests verify project listing, session display, filtering, and smart auto-enable behavior.

**Source:** [command/02_list.md](../../../../docs/cli/command/02_list.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| INT-1 | Default list shows all projects | Read Operations |
| INT-2 | type::uuid filters to UUID projects only | Filtering |
| INT-3 | type::path filters to path-encoded projects only | Filtering |
| INT-4 | show_sessions::1 expands session list per project | Session Display |
| INT-5 | path:: substring filters project list | Filtering |
| INT-6 | session:: auto-enables sessions display | Auto-Enable |
| INT-7 | agent::1 filters to agent sessions only | Filtering |
| INT-8 | agent::0 filters to main sessions only | Filtering |
| INT-9 | min_entries:: auto-enables sessions display | Auto-Enable |
| INT-10 | show_sessions::0 suppresses display even with session:: | Override |
| INT-11 | Combined path:: session:: filter | Filtering |
| INT-12 | Exit code 0 on empty storage | Exit Codes |

## Test Coverage Summary

- Read Operations: 1 test (INT-1)
- Filtering: 5 tests (INT-2, INT-3, INT-5, INT-7, INT-8)
- Session Display: 1 test (INT-4)
- Auto-Enable: 2 tests (INT-6, INT-9)
- Override: 1 test (INT-10)
- Filtering (combined): 1 test (INT-11)
- Exit Codes: 1 test (INT-12)

## Test Cases

---

### INT-1: Default list shows all projects

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .list
```

**Expected behavior:**
- Fixture: 3 projects named `alpha`, `beta`, `gamma`
- Three project entries in stdout, each showing a project path or identifier
- Exit code: 0
- **Source:** [command/02_list.md](../../../../docs/cli/command/02_list.md)

---

### INT-2: type::uuid filters to UUID projects only

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .list type::uuid
```

**Expected behavior:**
- Fixture: 1 UUID project `a1b2c3d4-...`, 2 path-encoded projects
- Only the 1 UUID project appears; path-encoded projects are absent
- Exit code: 0
- **Source:** [command/02_list.md](../../../../docs/cli/command/02_list.md)

---

### INT-3: type::path filters to path-encoded projects only

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .list type::path
```

**Expected behavior:**
- Fixture: 2 path-encoded projects, 1 UUID project
- Only the 2 path-encoded projects appear; the UUID project is absent
- Exit code: 0
- **Source:** [command/02_list.md](../../../../docs/cli/command/02_list.md)

---

### INT-4: show_sessions::1 expands session list per project

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .list show_sessions::1
```

**Expected behavior:**
- Fixture: 2 projects; project `alpha` has 2 sessions, project `beta` has 1 session
- Both projects listed, each followed by their respective session IDs (3 session entries total nested under their projects)
- Exit code: 0
- **Source:** [command/02_list.md](../../../../docs/cli/command/02_list.md)

---

### INT-5: path:: substring filters project list

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .list path::pro
```

**Expected behavior:**
- Fixture: projects at `/home/alice/projects/alpha`, `/home/alice/projects/beta`, `/tmp/other`
- Only projects whose path contains `pro` are listed (`alpha`, `beta`); `/tmp/other` is absent
- Exit code: 0
- **Source:** [command/02_list.md](../../../../docs/cli/command/02_list.md)

---

### INT-6: session:: auto-enables sessions display

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .list session::abc
```

**Expected behavior:**
- Fixture: 2 projects; session with ID containing `abc` exists in project `alpha`
- Sessions matching `abc` appear in output without requiring explicit `show_sessions::1`
- Exit code: 0
- **Source:** [command/02_list.md](../../../../docs/cli/command/02_list.md)

---

### INT-7: agent::1 filters to agent sessions only

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .list agent::1
```

**Expected behavior:**
- Fixture: project `alpha` with 1 agent session and 1 main session
- Only the agent session appears; the main session is not in output
- Exit code: 0
- **Source:** [command/02_list.md](../../../../docs/cli/command/02_list.md)

---

### INT-8: agent::0 filters to main sessions only

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .list agent::0
```

**Expected behavior:**
- Fixture: project `alpha` with 1 agent session and 1 main session
- Only the main session appears; the agent session is not in output
- Exit code: 0
- **Source:** [command/02_list.md](../../../../docs/cli/command/02_list.md)

---

### INT-9: min_entries:: auto-enables sessions display

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .list min_entries::10
```

**Expected behavior:**
- Fixture: project `alpha` has session `s1` with 15 entries and session `s2` with 3 entries
- Session `s1` (15 entries) appears; session `s2` (3 entries) does not
- Exit code: 0
- **Source:** [command/02_list.md](../../../../docs/cli/command/02_list.md)

---

### INT-10: show_sessions::0 suppresses display even with session::

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .list session::abc show_sessions::0
```

**Expected behavior:**
- Fixture: project `alpha` with a session whose ID contains `abc`
- Only project entries shown; no session entries appear despite `session::abc` being provided
- Exit code: 0
- **Source:** [command/02_list.md](../../../../docs/cli/command/02_list.md)

---

### INT-11: Combined path:: session:: filter

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .list path::pro session::abc
```

**Expected behavior:**
- Fixture: project `alpha` at path containing `pro` with session `s-abc`; project `other` at unrelated path with session `s-abc`
- Session `s-abc` under project `alpha` appears; session `s-abc` under project `other` does not (path filter excluded it)
- Exit code: 0
- **Source:** [command/02_list.md](../../../../docs/cli/command/02_list.md)

---

### INT-12: Exit code 0 on empty storage

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/empty-fixture clg .list
```

**Expected behavior:**
- Fixture: empty `projects/` directory, no project subdirectories
- Empty output or a "no projects found" message; no error
- Exit code: 0
- **Source:** [command/02_list.md](../../../../docs/cli/command/02_list.md)

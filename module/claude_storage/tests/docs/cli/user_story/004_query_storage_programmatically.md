# User Story :: 4. Query Storage Programmatically

Acceptance criteria tests for the developer persona querying storage from a script.
Source: [004_query_storage_programmatically.md](../../../../docs/cli/user_story/004_query_storage_programmatically.md)

## Test Case Index

| ID | Test Name | Criteria |
|----|-----------|---------|
| RWS-1 | list conversation count outputs bare integer | AC: count::1 outputs parseable integer |
| RWS-2 | count outputs bare integer | AC: .count outputs bare integer with no decorations |
| RWS-3 | count target:: specifies what to count | AC: can query count for specific target |
| RWS-4 | path:: scopes query to alternate storage root | AC: scope via path:: or CLAUDE_STORAGE_ROOT |
| RWS-5 | Non-existent storage root exits non-zero | AC: all commands exit 0 on success and non-zero on error |

---

### RWS-1: list conversation count outputs bare integer

**Scenario:** Script counts conversations in a specific project for reporting.

**Fixture:** 1 project with 3 conversations (root sessions).

**Command:**
```bash
clg .list type::conversation count::1 project::{project-id}
```

**Expected:**
- Stdout is a single bare integer (e.g., `3`)
- No headers, labels, or decorative characters
- Parseable by shell arithmetic: `[ $(clg .list type::conversation count::1 project::P) -gt 2 ]`

**Exit:** `0`

---

### RWS-2: count outputs bare integer

**Scenario:** Script checks session count for a threshold alert.

**Fixture:** 3 projects in storage.

**Command:**
```bash
clg .count
```

**Expected:**
- Stdout is a single integer (e.g., `3`)
- No labels or decorators
- Usable directly in shell arithmetic: `[ $(clg .count) -gt 2 ]`

**Exit:** `0`

---

### RWS-3: count target specifies what to count

**Scenario:** Script needs a session count rather than project count.

**Fixture:** 2 projects; project A has 3 sessions, project B has 2 sessions.

**Command:**
```bash
clg .count target::sessions
```

**Expected:**
- Stdout is `5` (total sessions across all projects)

**Exit:** `0`

---

### RWS-4: path:: scopes query to alternate storage root

**Scenario:** Script monitors a secondary storage location for capacity.

**Fixture:** Alternate root at `/tmp/clg-alt-{ts}/` with 1 project and 1 session.

**Command:**
```bash
clg .count path::/tmp/clg-alt-{ts}
```

**Expected:**
- Stdout is `1` (project count from the alternate root, not the default)

**Exit:** `0`

---

### RWS-5: Non-existent storage root exits non-zero

**Scenario:** Script handles the case where storage is not accessible.

**Fixture:** None — the path does not exist.

**Command:**
```bash
clg .status path::/tmp/does-not-exist-clg-test
```

**Expected:**
- Exit code is `2` (storage read error)
- Stderr contains an error message about the inaccessible path

**Exit:** `2`

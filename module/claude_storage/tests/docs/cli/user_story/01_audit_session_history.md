# User Story :: 1. Audit Session History

Acceptance criteria tests for the developer persona auditing Claude Code session storage.
Source: [001_audit_session_history.md](../../../../docs/cli/user_story/001_audit_session_history.md)

## Test Case Index

| ID | Test Name | Criteria |
|----|-----------|---------|
| RWS-1 | Basic status shows project and session totals | AC: view total project and session count |
| RWS-2 | Verbosity 2 shows per-project breakdown | AC: drill into per-project session counts |
| RWS-3 | Verbosity 0 outputs machine-readable format | AC: verbosity::0 suitable for scripts |
| RWS-4 | Count target::sessions returns bare integer | AC: count specific targets independently |
| RWS-5 | Path override inspects alternate storage root | AC: override storage root |

---

### RWS-1: Basic status shows project and session totals

**Scenario:** Developer runs `.status` to get an at-a-glance overview of storage.

**Fixture:** At least 2 projects with 1 session each in `CLAUDE_STORAGE_ROOT`.

**Command:**
```bash
clg .status
```

**Expected:**
- Stdout contains a summary with project count and session count
- Output includes the storage root path

**Exit:** `0`

---

### RWS-2: Verbosity 2 shows per-project breakdown

**Scenario:** Developer drills into per-project session counts.

**Fixture:** 2 projects, each with 1 session containing at least 2 entries.

**Command:**
```bash
clg .status verbosity::2
```

**Expected:**
- Stdout includes per-project session counts
- Stdout includes entry breakdowns per project
- More detail than `verbosity::1` output

**Exit:** `0`

---

### RWS-3: Verbosity 0 outputs machine-readable format

**Scenario:** Developer pipes `.status` output to a script.

**Fixture:** At least 1 project with 1 session.

**Command:**
```bash
clg .status verbosity::0
```

**Expected:**
- Stdout matches `projects: N, sessions: N` (parseable key=value)
- No decorative headers or tables

**Exit:** `0`

---

### RWS-4: Count target sessions returns bare integer

**Scenario:** Developer needs an exact session count for threshold checking.

**Fixture:** 3 sessions across 2 projects.

**Command:**
```bash
clg .count target::sessions
```

**Expected:**
- Stdout is a single bare integer (e.g., `3`)
- No labels, headers, or trailing text beyond the newline

**Exit:** `0`

---

### RWS-5: Path override inspects alternate storage root

**Scenario:** Developer inspects a backup storage location.

**Fixture:** Separate fixture at `/tmp/clg-alt-{ts}/` with 1 project and 2 sessions.

**Command:**
```bash
clg .status path::/tmp/clg-alt-{ts}
```

**Expected:**
- Stdout reflects the alternate storage root contents (not the default `~/.claude/`)
- Project and session counts match the alternate fixture

**Exit:** `0`

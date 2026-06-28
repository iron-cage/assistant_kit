# User Story :: 1. Audit Session History

Acceptance criteria tests for the developer persona auditing Claude Code session storage.
Source: [001_audit_session_history.md](../../../../docs/cli/user_story/001_audit_session_history.md)

## Test Case Index

| ID | Test Name | Criteria |
|----|-----------|---------|
| RWS-1 | Basic status shows project and session totals | AC: view total project and session count |
| RWS-2 | show_tokens reveals token consumption | AC: view token usage for audit |
| RWS-3 | List with sessions shows per-project history | AC: drill into per-project session listing |
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

### RWS-2: show_tokens reveals token consumption

**Scenario:** Developer audits token usage across all sessions.

**Fixture:** At least 1 project with 1 session containing entries with token data.

**Command:**
```bash
clg .status show_tokens::1
```

**Expected:**
- Stdout includes standard project/session counts
- Stdout includes Tokens section with Input, Output, Cache Read, Cache Creation values

**Exit:** `0`

---

### RWS-3: List with sessions shows per-project history

**Scenario:** Developer drills into per-project session listing for audit.

**Fixture:** 2 projects, each with at least 1 session.

**Command:**
```bash
clg .list show_sessions::1
```

**Expected:**
- Stdout lists each project with its sessions underneath
- Session IDs are visible for further inspection

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

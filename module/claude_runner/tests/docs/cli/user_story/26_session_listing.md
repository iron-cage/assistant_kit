# User Story :: Session Listing

Test case spec for [026_session_listing.md](../../../../docs/cli/user_story/026_session_listing.md).

## Test Case Index

| ID | Test Name | AC | Status |
|----|-----------|-----|----|
| US-1 | No sessions: exit 0, no-sessions message | AC-002 | ✅ |
| US-2 | Help lists `ps` subcommand | AC-003 | ✅ |
| US-3 | Typo `clr p` triggers guard | AC-004 | ✅ |
| US-4 | Sessions present: unicode-box table with correct headers | AC-001, AC-005 | ✅ |

---

### US-1: No sessions — no-sessions message

- **Given:** No `claude` processes running (test container has 0 sessions)
- **When:** `clr ps`
- **Then:** Exit 0; stdout = `No active Claude Code sessions.`; stdout does not contain `┌`
- **Exit:** 0
- **Verifies:** AC-002

---

### US-2: Help lists `ps`

- **Given:** Developer wants to discover available subcommands
- **When:** `clr --help`
- **Then:** Exit 0; stdout contains `ps`
- **Exit:** 0
- **Verifies:** AC-003

---

### US-3: Typo guard for `clr p`

- **Given:** Developer miskeys `clr ps` as `clr p`
- **When:** `clr p`
- **Then:** Exit 1; stderr contains `Did you mean`
- **Exit:** 1
- **Verifies:** AC-004

---

### US-4: Sessions present — unicode-box table with headers

- **Given:** ≥1 fake `claude` process running; PATH prepended with fake dir
- **When:** `clr ps`
- **Then:** Exit 0; stdout contains `┌` and `PID` and `Absolute Path` and `Task`
- **Exit:** 0
- **Verifies:** AC-001, AC-005

# User Story :: Session Termination

Test case spec for [027_session_termination.md](../../../../docs/cli/user_story/027_session_termination.md).

## Test Case Index

| ID | Test Name | AC | Status |
|----|-----------|-----|-----|
| US-1 | Successful kill: exit 0, "Sent SIGTERM" message | AC-001 | ✅ |
| US-2 | Non-Claude PID: exit 1, not-a-session error | AC-002 | ✅ |
| US-3 | No PID: exit 1, missing PID error | AC-003 | ✅ |
| US-4 | `clr kill --help` shows usage with SIGTERM | AC-004 | ✅ |
| US-5 | `clr --help` lists `kill` subcommand | AC-005 | ✅ |
| US-6 | Typo `clr kil` triggers guard | AC-006 | ✅ |

---

### US-1: Successful kill — SIGTERM delivered

- **Given:** A fake `claude` ELF process is running (PID obtained from spawn)
- **When:** `clr kill <pid>`
- **Then:** Exit 0; stdout contains `"Sent SIGTERM to Claude Code session <PID>."`; process terminates
- **Exit:** 0
- **Verifies:** AC-001
- **Platform:** Linux/Unix only (`#[cfg(unix)]`)

---

### US-2: Non-Claude PID rejected

- **Given:** A PID that is not a running `claude` process (e.g. `999999`)
- **When:** `clr kill 999999`
- **Then:** Exit 1; stderr contains `"999999"` and `"not a running Claude Code session"`
- **Exit:** 1
- **Verifies:** AC-002

---

### US-3: Missing PID

- **Given:** Developer omits the PID
- **When:** `clr kill`
- **Then:** Exit 1; stderr contains `"missing PID"`
- **Exit:** 1
- **Verifies:** AC-003

---

### US-4: `clr kill --help`

- **Given:** Developer wants usage information
- **When:** `clr kill --help`
- **Then:** Exit 0; stdout contains `"SIGTERM"` and `"<PID>"`
- **Exit:** 0
- **Verifies:** AC-004

---

### US-5: Help lists `kill`

- **Given:** Developer wants to discover available subcommands
- **When:** `clr --help`
- **Then:** Exit 0; stdout contains `kill`
- **Exit:** 0
- **Verifies:** AC-005

---

### US-6: Typo guard for `clr kil`

- **Given:** Developer miskeys `clr kill` as `clr kil`
- **When:** `clr kil`
- **Then:** Exit 1; stderr contains `"Did you mean"`
- **Exit:** 1
- **Verifies:** AC-006

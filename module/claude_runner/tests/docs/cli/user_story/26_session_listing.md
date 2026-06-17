# User Story :: Session Listing

Test case spec for [026_session_listing.md](../../../../docs/cli/user_story/026_session_listing.md).

## Test Case Index

| ID | Test Name | AC | Status |
|----|-----------|-----|----|
| US-1 | No sessions: exit 0, no-sessions message | AC-002 | ✅ |
| US-2 | Help lists `ps` subcommand | AC-003 | ✅ |
| US-3 | Typo `clr p` triggers guard | AC-004 | ✅ |
| US-4 | Sessions present: plain-style table with correct headers | AC-001, AC-005 | ✅ |
| US-5 | `$PRO` prefix replaced by `"$PRO"` literal in Absolute Path column | AC-007 | ✅ |
| US-6 | Queued CLR session shown when gate file present | AC-008 | ✅ |
| US-7 | Active table caption contains `Active Sessions` and count suffix | AC-010 | ✅ |
| US-8 | `clr ps --help` prints help and exits 0 | AC-011 | ✅ |
| US-9 | Active sessions ordered oldest first | AC-012 | ✅ |

---

### US-1: No sessions — no-sessions message

- **Given:** No `claude` processes visible — empty temp dir set as `CLR_PROC_DIR` so `find_claude_processes()` returns no results regardless of host sessions
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

### US-4: Sessions present — plain-style table with headers

- **Given:** ≥1 fake `claude` process running; PATH prepended with fake dir
- **When:** `clr ps`
- **Then:** Exit 0; stdout contains `PID`, `Elapsed`, `Absolute Path`, and `Task`; stdout does NOT contain `┌` (plain style)
- **Exit:** 0
- **Verifies:** AC-001, AC-005

---

### US-5: `$PRO` prefix shortened in Absolute Path column

- **Given:** temp dir as `$PRO` root; subdirectory `workspace` within it; fake `claude` ELF spawned in that subdir; `PRO` set to temp dir when running `clr ps`
- **When:** `clr ps` with `PRO=<temp_dir>` in env
- **Then:** Exit 0; stdout contains `"$PRO"`; stdout does NOT contain the full temp dir path
- **Exit:** 0
- **Verifies:** AC-007

---

### US-6: Queued CLR session shown when gate file present

- **Given:** temp dir used as `CLR_GATE_DIR`; a gate JSON file written at `<temp_dir>/{test_process_pid}.json` with `cwd`, `since`, `attempt`, `message` fields. Uses the test process's own PID so the `/proc/{pid}` liveness filter passes (gate files with dead PIDs are filtered out per BUG-293)
- **When:** `clr ps` with `CLR_GATE_DIR=<temp_dir>` in env
- **Then:** Exit 0; stdout contains `PID`, `CWD`, and `Waiting` (queued table headers)
- **Exit:** 0
- **Verifies:** AC-008

---

### US-7: Active table caption contains `Active Sessions` and count suffix

- **Given:** ≥1 fake `claude` process running; PATH prepended with fake dir
- **When:** `clr ps`
- **Then:** Exit 0; stdout contains `Active Sessions` (caption title) and `running` (count suffix from the caption rule line above the column headers)
- **Exit:** 0
- **Verifies:** AC-010

---

### US-8: `clr ps --help` prints help

- **Given:** Developer wants documentation for the `ps` subcommand
- **When:** `clr ps --help`
- **Then:** Exit 0; stdout contains help text (e.g. column descriptions or `Active Sessions`)
- **Exit:** 0
- **Verifies:** AC-011

---

### US-9: Active sessions ordered oldest first

- **Given:** ≥2 fake `claude` processes running with different start times; PATH prepended with fake dir
- **When:** `clr ps`
- **Then:** Exit 0; the row at `#1` has the longest elapsed time; row `#2` has a shorter elapsed time than row `#1`
- **Exit:** 0
- **Verifies:** AC-012
- **Note:** In typical Linux environments PID allocation is monotonic, so spawn-order correlates with PID order. This test validates that ordering is applied but cannot distinguish age-sort from PID-sort without `/proc` mocking. The behavioral guarantee is: oldest session appears first.

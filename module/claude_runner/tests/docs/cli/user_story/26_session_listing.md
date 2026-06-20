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
| US-10 | `--mode print` shows only print-mode sessions | AC-013 | ✅ |
| US-11 | `--mode bogus` exits 1 with error | AC-014 | ✅ |
| US-12 | `--columns pid,path,task` shows custom column subset | AC-015 | ✅ |
| US-13 | `--columns bogus` exits 1 with error | AC-016 | ✅ |
| US-14 | `--wide` shows all 11 columns | AC-017 | ✅ |
| US-15 | `--columns` overrides `--wide` | AC-018 | ✅ |
| US-16 | `CLR_PS_MODE` env var filters sessions | AC-019 | ✅ |
| US-17 | `CLR_PS_COLUMNS` env var selects columns | AC-020 | ✅ |
| US-18 | `Flags` column absent when no flags apply to any session | AC-021 | ✅ |
| US-19 | 🐳 Container flag appears for session cwd outside `$HOME` | AC-023 | ✅ |
| US-20 | 🕰 Ancient flag with `CLR_PS_ANCIENT_SECS=0` threshold | AC-024 | ✅ |
| US-21 | 🐘 High-RAM flag with `CLR_PS_HIGH_RAM_MB=0` threshold | AC-025 | ✅ |
| US-22 | ⚠ Dead-metrics flag for session with unreadable proc stats | AC-026 | ✅ |
| US-23 | ⚡ Running flag for session in kernel state R | AC-027 | ✅ |
| US-24 | 🖨 Print-mode flag for print-mode session | AC-028 | ✅ |
| US-25 | Legend appears below active table when flags present | AC-030 | ✅ |
| US-26 | Legend absent when no flags present | AC-030 | ✅ |

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

---

### US-10: `--mode print` shows only print-mode sessions

- **Given:** 2 fake `claude` processes running: one with `--print` arg, one without
- **When:** `clr ps --mode print`
- **Then:** Exit 0; output contains only the print-mode session PID
- **Exit:** 0
- **Verifies:** AC-013

---

### US-11: `--mode bogus` exits 1

- **Given:** Developer passes an invalid mode value
- **When:** `clr ps --mode bogus`
- **Then:** Exit 1; stderr contains error with valid values
- **Exit:** 1
- **Verifies:** AC-014

---

### US-12: `--columns` selects custom subset

- **Given:** ≥1 fake `claude` process running
- **When:** `clr ps --columns pid,path,task`
- **Then:** Exit 0; stdout contains `PID`, `Absolute Path`, `Task`; does NOT contain `CPU%`, `RAM`
- **Exit:** 0
- **Verifies:** AC-015

---

### US-13: `--columns bogus` exits 1

- **Given:** Developer passes an invalid column key
- **When:** `clr ps --columns bogus`
- **Then:** Exit 1; stderr contains error listing valid keys
- **Exit:** 1
- **Verifies:** AC-016

---

### US-14: `--wide` shows all 11 columns

- **Given:** ≥1 fake `claude` process running
- **When:** `clr ps --wide`
- **Then:** Exit 0; stdout contains `Mode`, `Command`, `Binary` (plus 9 defaults including Mode)
- **Exit:** 0
- **Verifies:** AC-017

---

### US-15: `--columns` overrides `--wide`

- **Given:** ≥1 fake `claude` process running
- **When:** `clr ps --wide --columns pid,task`
- **Then:** Exit 0; stdout contains `PID`, `Task`; does NOT contain `Mode`, `Command`, `Binary`
- **Exit:** 0
- **Verifies:** AC-018

---

### US-16: `CLR_PS_MODE` env var filters sessions

- **Given:** 2 fake `claude` processes running: one print-mode, one interactive
- **When:** `clr ps` with `CLR_PS_MODE=print` in env
- **Then:** Exit 0; output contains only the print-mode session PID
- **Exit:** 0
- **Verifies:** AC-019

---

### US-17: `CLR_PS_COLUMNS` env var selects columns

- **Given:** ≥1 fake `claude` process running
- **When:** `clr ps` with `CLR_PS_COLUMNS=pid,elapsed` in env
- **Then:** Exit 0; stdout contains `PID`, `Elapsed`; does NOT contain `CPU%`, `RAM`, `Task`
- **Exit:** 0
- **Verifies:** AC-020

---

### US-18: Flags column absent when no flags apply

- **Given:** Fake `claude` ELF running in a subdirectory of `$HOME`; `CLR_PS_ANCIENT_SECS=999999`; `CLR_PS_HIGH_RAM_MB=999999`; interactive mode
- **When:** `clr ps`
- **Then:** Exit 0; stdout does NOT contain `Flags` header; no flag emoji in output
- **Exit:** 0
- **Verifies:** AC-021

---

### US-19: 🐳 Container flag for session cwd outside `$HOME`

- **Given:** Fake `claude` ELF spawned with cwd `/tmp/workspace` (outside temp HOME); `HOME=<temp_home>`
- **When:** `clr ps`
- **Then:** Exit 0; stdout contains `Flags` column header; the session row contains `🐳`; legend below active table lists `🐳  Container`
- **Exit:** 0
- **Verifies:** AC-023

---

### US-20: 🕰 Ancient flag with `CLR_PS_ANCIENT_SECS=0`

- **Given:** Fake `claude` ELF running; `CLR_PS_ANCIENT_SECS=0` (threshold = 0 → any elapsed time triggers flag)
- **When:** `clr ps` with `CLR_PS_ANCIENT_SECS=0` in env
- **Then:** Exit 0; stdout contains `🕰`; legend lists `🕰  Ancient`
- **Exit:** 0
- **Verifies:** AC-024

---

### US-21: 🐘 High-RAM flag with `CLR_PS_HIGH_RAM_MB=0`

- **Given:** Fake `claude` ELF running; `CLR_PS_HIGH_RAM_MB=0` (threshold = 0 → any non-zero RSS triggers flag)
- **When:** `clr ps` with `CLR_PS_HIGH_RAM_MB=0` in env
- **Then:** Exit 0; stdout contains `🐘`; legend lists `🐘  High RAM`
- **Exit:** 0
- **Verifies:** AC-025

---

### US-22: ⚠ Dead-metrics flag for session with unreadable proc stats

- **Given:** Fake `claude` cmdline entry visible in proc scan but `/proc/{pid}/stat` absent (TOCTOU-dead session)
- **When:** `clr ps`
- **Then:** Exit 0; stdout contains `⚠`; legend lists `⚠  Dead metrics`
- **Exit:** 0
- **Verifies:** AC-026

---

### US-23: ⚡ Running flag for session in kernel state R

- **Given:** Fake `claude` process whose `/proc/{pid}/stat` state field is `R`
- **When:** `clr ps`
- **Then:** Exit 0; stdout contains `⚡`; legend lists `⚡  Running`
- **Exit:** 0
- **Verifies:** AC-027
- **Note:** Kernel state `R` may occur naturally for CPU-intensive processes or be synthesised via a fake proc dir

---

### US-24: 🖨 Print-mode flag for print-mode session

- **Given:** Fake `claude` ELF spawned with `--print` arg; cwd inside `$HOME`; `CLR_PS_ANCIENT_SECS=999999`; `CLR_PS_HIGH_RAM_MB=999999`
- **When:** `clr ps`
- **Then:** Exit 0; stdout contains `🖨`; legend lists `🖨  Print mode`
- **Exit:** 0
- **Verifies:** AC-028

---

### US-25: Legend appears when flags present

- **Given:** Fake `claude` ELF running with 🐳 flag (cwd outside `$HOME`)
- **When:** `clr ps`
- **Then:** Exit 0; stdout contains a single legend line after the active sessions table; legend line contains `🐳` and `Container`
- **Exit:** 0
- **Verifies:** AC-030

---

### US-26: Legend absent when no flags present

- **Given:** Fake `claude` ELF running inside `$HOME`; `CLR_PS_ANCIENT_SECS=999999`; `CLR_PS_HIGH_RAM_MB=999999`; interactive mode
- **When:** `clr ps`
- **Then:** Exit 0; stdout does NOT contain any flag emoji (`👈`, `🖨`, `⚡`, `🕰`, `🐘`, `⚠`, `🐳`)
- **Exit:** 0
- **Verifies:** AC-030

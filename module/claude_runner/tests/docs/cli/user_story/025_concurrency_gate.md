# User Story :: Session Concurrency Gate

Test case spec for [025_concurrency_gate.md](../../../../docs/cli/user_story/025_concurrency_gate.md).

## Test Case Index

| ID | Test Name | AC | Status |
|----|-----------|-----|----|
| US-1 | Gate disabled: `--max-sessions 0` proceeds immediately with no messages | AC-004 | ✅ |
| US-2 | Env-var fallback: `CLR_MAX_SESSIONS=N` equivalent to `--max-sessions N` | AC-005 | ✅ |
| US-3 | CLI wins: `--max-sessions M` beats `CLR_MAX_SESSIONS=N` | AC-005 | ✅ |
| US-4 | Dry-run bypass: gate not triggered in `--dry-run` mode | AC-006 | ✅ |
| US-5 | Interactive invocations bypass the gate entirely, regardless of active session count | AC-007 | ✅ |
| US-6 | Gate's active-session count excludes interactive sessions (counts only non-interactive processes) | AC-008 | ✅ |

---

### US-1: Gate disabled — proceeds immediately with no messages

- **Given:** Developer sets `--max-sessions 0`; any number of active Claude processes present
- **When:** `clr --max-sessions 0 --dry-run "task"`
- **Then:** Exit 0; command produces output immediately; no "waiting" or timeout messages on stderr
- **Exit:** 0
- **Verifies:** AC-004

---

### US-2: CLR_MAX_SESSIONS env-var fallback

- **Given:** Developer sets `CLR_MAX_SESSIONS=3`; no `--max-sessions` CLI flag; `--dry-run` set to avoid live execution
- **When:** `CLR_MAX_SESSIONS=3 clr --dry-run "task"`
- **Then:** Exit 0; env var applied (verified via env_var_test framework); gate would use 3 as the limit in a live run; dry-run produces output immediately
- **Exit:** 0
- **Verifies:** AC-005

---

### US-3: CLI wins over CLR_MAX_SESSIONS

- **Given:** Developer sets `CLR_MAX_SESSIONS=1` (very strict); overrides via `--max-sessions 10` on CLI; `--dry-run` set
- **When:** `CLR_MAX_SESSIONS=1 clr --max-sessions 10 --dry-run "task"`
- **Then:** Exit 0; CLI value 10 used (env var 1 ignored); verified via env_var_test framework
- **Exit:** 0
- **Verifies:** AC-005 (CLI-wins)

---

### US-4: Dry-run bypasses session gate

- **Given:** Developer uses `--dry-run` to preview the command; any `--max-sessions` value set
- **When:** `clr --max-sessions 1 --dry-run "task"` (max sessions = 1; even if 1+ active)
- **Then:** Exit 0; command preview printed immediately; no waiting messages on stderr
- **Exit:** 0
- **Verifies:** AC-006

---

### US-5: Interactive invocations bypass the gate entirely

- **Given:** Developer runs `clr` in `--interactive` mode; `--max-sessions 1` set; 15 fake print-mode + 1 fake interactive Claude process active (via `$CLR_PROC_DIR` fixture)
- **When:** `clr --interactive --max-sessions 1 "task"`
- **Then:** Exit 0; command proceeds immediately; no "waiting" or gate messages on stderr, regardless of active session count
- **Exit:** 0
- **Verifies:** AC-007
- **Test:** `t03_interactive_invocation_bypasses_gate_with_zero_wait` (`concurrency_gate_test.rs`) — proves zero wait via wall-clock elapsed time (AF1), not just message absence

---

### US-6: Gate count excludes interactive sessions

- **Given:** `--max-sessions 5` set; 10 fake interactive Claude processes active and 5 fake print-mode Claude processes active (via `$CLR_PROC_DIR` fixture); print-mode invocation issued
- **When:** `clr -p --max-sessions 5 "task"` (10 interactive + 5 print-mode active; print-mode count = 5, at the limit)
- **Then:** Exit 0; gate triggers at exactly `5/5` (not `15/5`); interactive fake processes excluded from the count; releases once a short-lived print-mode process self-expires
- **Exit:** 0
- **Verifies:** AC-008
- **Test:** `t04_gate_counts_print_mode_only_excludes_interactive` (`concurrency_gate_test.rs`) — anchored `"Info: 5/5"` assertion prevents an unfiltered `15/5` count from false-passing (AF1)
- **Note:** Boundary/regression coverage for the same print-mode-only counting behavior — gate triggers at exactly the limit and not below it — is additionally covered by `t01_gate_triggers_at_six_print_mode_processes` and `t02_gate_does_not_trigger_below_six_print_mode_processes`; `t06_max_sessions_zero_disables_gate_regardless_of_count` regression-guards US-1 against the print-mode-only counting change (all three in `concurrency_gate_test.rs`)

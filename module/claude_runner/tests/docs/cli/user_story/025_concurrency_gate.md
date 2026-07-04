# User Story :: Session Concurrency Gate

Test case spec for [025_concurrency_gate.md](../../../../docs/cli/user_story/025_concurrency_gate.md).

## Test Case Index

| ID | Test Name | AC | Status |
|----|-----------|-----|----|
| US-1 | Gate disabled: `--max-sessions 0` proceeds immediately with no messages | AC-004 | ✅ |
| US-2 | Env-var fallback: `CLR_MAX_SESSIONS=N` equivalent to `--max-sessions N` | AC-005 | ✅ |
| US-3 | CLI wins: `--max-sessions M` beats `CLR_MAX_SESSIONS=N` | AC-005 | ✅ |
| US-4 | Dry-run bypass: gate not triggered in `--dry-run` mode | AC-006 | ✅ |

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

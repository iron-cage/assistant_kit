# Parameter :: `--max-sessions`

Edge case coverage for the `--max-sessions` parameter. See [033_max_sessions.md](../../../../docs/cli/param/033_max_sessions.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--max-sessions` | Documentation |
| EC-2 | `--max-sessions 0` + `--dry-run` → exit 0; no gate messages | Behavioral Divergence |
| EC-3 | `CLR_MAX_SESSIONS=5` + `--dry-run` → exit 0 (env var applied; gate skipped in dry-run) | Env Var |
| EC-4 | `--max-sessions 5` + `CLR_MAX_SESSIONS=2` + `--dry-run` → CLI 5 wins; exit 0 | CLI-wins |
| EC-5 | `CLR_MAX_SESSIONS=notanumber` → silently ignored; default 6 used; command proceeds | Validation |
| EC-6 | `--max-sessions 0` → gate disabled; no stderr waiting messages emitted | Behavioral |
| EC-7 | No gate messages when sessions below limit (dry-run, default max) | Behavioral Divergence |
| EC-8 | Gate disabled with explicit 0 → no stderr messages | Edge Case |
| EC-9 | `--help` output contains `default: 6` for `--max-sessions` | Documentation |

## Test Coverage Summary

- Documentation: 2 tests (EC-1, EC-9)
- Behavioral Divergence: 2 tests (EC-2, EC-7)
- Env Var: 1 test (EC-3)
- CLI-wins: 1 test (EC-4)
- Validation: 1 test (EC-5)
- Behavioral: 1 test (EC-6)
- Edge Case: 1 test (EC-8)

**Total:** 9 edge cases

## Architectural Constraint

The gate-triggered behavior (blocking + stderr waiting messages when sessions ≥ limit) cannot
be demonstrated in integration tests without live Claude processes. EC-2 and EC-7 represent the
maximum testable behavioral divergence: `--max-sessions 0` bypasses `find_claude_processes()`
entirely (gate disabled at configuration level), while any value > 0 enters the gate code path
and invokes `find_claude_processes()` before each subprocess launch. True blocking behavior and
timeout exit-1 are validated by the parameter specification (`033_max_sessions.md`) and cannot
be exercised in this test surface.

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_max_sessions_help_listed` | `param_edge_cases_test.rs` |
| EC-2 | `us25_1_max_sessions_0_unlimited_no_wait` | `user_story_output_test.rs` |
| EC-3 | `us25_2_clr_max_sessions_env_var_applied`, `e30_clr_max_sessions_accepted_in_dry_run` | `user_story_output_test.rs`, `env_var_ext_test.rs` |
| EC-4 | `us25_3_cli_max_sessions_wins_over_env`, `e30_clr_max_sessions_accepted_in_dry_run` | `user_story_output_test.rs`, `env_var_ext_test.rs` |
| EC-5 | `e30_clr_max_sessions_accepted_in_dry_run` (invalid value sub-assertion) | `env_var_ext_test.rs` |
| EC-6 | `us25_1_max_sessions_0_unlimited_no_wait` | `user_story_output_test.rs` |
| EC-7 | `ec7_max_sessions_no_gate_messages_below_limit` | `param_edge_cases_test.rs` |
| EC-8 | `us25_1_max_sessions_0_unlimited_no_wait` | `user_story_output_test.rs` |
| EC-9 | `ec9_max_sessions_help_shows_default_six` | `param_edge_cases_test.rs` |

---

### EC-1: --help lists --max-sessions

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--max-sessions`
- **Exit:** 0
- **Source:** [command/01_run.md](../../../../docs/cli/command/01_run.md)
- **Commands:** run, ask

---

### EC-2: --max-sessions 0 + --dry-run → exit 0; no gate messages

- **Given:** `--max-sessions 0` and `--dry-run` set; any number of active Claude processes
- **When:** `clr --max-sessions 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced; no waiting or timeout messages on stderr
- **Exit:** 0
- **Source:** [--max-sessions](../../../../docs/cli/param/033_max_sessions.md)
- **Commands:** run, ask

---

### EC-3: CLR_MAX_SESSIONS=5 env var → applied when CLI flag absent

- **Given:** `CLR_MAX_SESSIONS=5` set; no `--max-sessions` CLI flag; `--dry-run` set
- **When:** `CLR_MAX_SESSIONS=5 clr --dry-run "task"`
- **Then:** Exit 0; dry-run output produced immediately (gate skipped in dry-run regardless of env var value)
- **Exit:** 0
- **Source:** [--max-sessions](../../../../docs/cli/param/033_max_sessions.md)
- **Commands:** run, ask

---

### EC-4: --max-sessions CLI wins over CLR_MAX_SESSIONS env var

- **Given:** `CLR_MAX_SESSIONS=2` set; `--max-sessions 5` on CLI; `--dry-run` set
- **When:** `CLR_MAX_SESSIONS=2 clr --max-sessions 5 --dry-run "task"`
- **Then:** Exit 0; CLI value 5 used (env var 2 ignored); dry-run output produced
- **Exit:** 0
- **Source:** [--max-sessions](../../../../docs/cli/param/033_max_sessions.md)
- **Commands:** run, ask

---

### EC-5: CLR_MAX_SESSIONS=invalid → silently ignored; default 6 used

- **Given:** `CLR_MAX_SESSIONS=notanumber` set; no `--max-sessions` CLI flag; `--dry-run` set
- **When:** `CLR_MAX_SESSIONS=notanumber clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored; default 6 used (gate skipped in dry-run)
- **Exit:** 0
- **Source:** [--max-sessions](../../../../docs/cli/param/033_max_sessions.md)
- **Commands:** run, ask

---

### EC-6: --max-sessions 0 → no stderr gate messages

- **Given:** `--max-sessions 0` set; `--dry-run` used to avoid live subprocess
- **When:** `clr --max-sessions 0 --dry-run "task"`
- **Then:** Exit 0; no "Waiting for session slot", no timeout warning, no "proceeding" message on stderr
- **Exit:** 0
- **Source:** [--max-sessions](../../../../docs/cli/param/033_max_sessions.md)
- **Commands:** run, ask

---

### EC-7: No gate messages when sessions below limit (dry-run, default max)

- **Given:** clean environment; no `--max-sessions` override; real session count is 0 (no claude processes running)
- **When:** `clr --dry-run "task"` (default max=6; 0 active sessions; gate not triggered)
- **Then:** Exit 0; no "waiting" or "session" messages on stderr; command preview produced immediately. **Divergence from EC-2:** value 6 activates the gate code path — in non-dry-run execution `find_claude_processes()` would be called (finds 0 < 6, proceeds); value 0 (EC-2) bypasses `find_claude_processes()` entirely regardless of mode, as a configuration-level disable
- **Exit:** 0
- **Source:** [--max-sessions](../../../../docs/cli/param/033_max_sessions.md)
- **Commands:** run, ask

---

### EC-8: Gate disabled with explicit 0 → no stderr messages

- **Given:** `--max-sessions 0` and `--dry-run` set
- **When:** `clr --max-sessions 0 --dry-run "task"`
- **Then:** Exit 0; no "waiting", "session", or "limit" messages on stderr regardless of active process count
- **Exit:** 0
- **Source:** [--max-sessions](../../../../docs/cli/param/033_max_sessions.md)
- **Commands:** run, ask

---

### EC-9: --help shows correct default (6) for --max-sessions

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--max-sessions` AND contains `default: 6`; prevents regression where help text shows stale default while code uses a different value
- **Exit:** 0
- **Source:** [--max-sessions](../../../../docs/cli/param/033_max_sessions.md)
- **Commands:** run, ask

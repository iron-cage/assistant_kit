# Test: Invariant — Print-Mode Timeout Default

Test case planning for [invariant/007_print_mode_timeout.md](../../../docs/invariant/007_print_mode_timeout.md). Tests validate that `run_print_mode()` applies the `DEFAULT_PRINT_TIMEOUT_SECS = 3600` watchdog when no explicit `--timeout` is given, and that `run_interactive()` remains unbounded.

**Source:** [invariant/007_print_mode_timeout.md](../../../docs/invariant/007_print_mode_timeout.md)
**Related:** [cli/param/036_timeout.md](../cli/param/036_timeout.md), [invariant/006_exit_codes.md](006_exit_codes.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `DEFAULT_PRINT_TIMEOUT_SECS` constant equals `3600` in source | Structural |
| IT-2 | No `--timeout`, fast subprocess → exit 0, no timeout message (default does not fire early) | Invariant Hold |
| IT-3 | No `--timeout`, 2s subprocess → exit 0 in ≤ 10s (3600s watchdog does not fire before subprocess) | Invariant Hold |
| IT-4 | `--timeout 7200` with fast subprocess → explicit value wins over 3600 default | Explicit Override |
| IT-5 | `--timeout 0` opts out of 3600s default; fast subprocess → exit 0 (unlimited) | Explicit Override |
| IT-6 | `CLR_TIMEOUT=0` opts out of 3600s default; fast subprocess → exit 0 (unlimited) | Env Var Override |
| IT-7 | `CLR_TIMEOUT=3600` accepted without error; dry-run exits 0 | Env Var |
| IT-8 | No `--timeout`; `_CLR_DEFAULT_TIMEOUT=2`; hanging subprocess → exit 4, killed by default watchdog | Invariant Kill |

## Test Coverage Summary

- Structural: 1 test (IT-1)
- Invariant Hold: 2 tests (IT-2, IT-3)
- Invariant Kill: 1 test (IT-8)
- Explicit Override: 2 tests (IT-4, IT-5)
- Env Var Override: 1 test (IT-6)
- Env Var: 1 test (IT-7)

**Total:** 8 invariant test cases

## Architectural Constraint

IT-2 and IT-3 use a fake `claude` subprocess to avoid live API calls. IT-3 uses a 2-second sleep script — just long enough to verify the 3600s default does not fire, but short enough not to actually wait 3600s. IT-4 through IT-7 confirm the explicit-override path (`Some(n).unwrap_or(3600) = n`). IT-8 uses a test-only internal env var `_CLR_DEFAULT_TIMEOUT` that overrides `DEFAULT_PRINT_TIMEOUT_SECS` to a short value (2s). This allows the test to verify the default-path kill mechanism without waiting 3600 seconds. The env var is prefixed with `_` to signal internal/test-only use and is not documented in user-facing param docs. All 8 tests are fully automated in `timeout_test.rs` and `env_var_test.rs`; no live claude is needed.

## Implementation Notes

| IT | Test Function | File |
|----|---------------|------|
| IT-1 | `ec_timeout_default_constant_value` | `tests/timeout_test.rs` |
| IT-2 | `ec_timeout_default_no_fire` | `tests/timeout_test.rs` |
| IT-3 | `ec_timeout_default_activates_watchdog` | `tests/timeout_test.rs` |
| IT-4 | `ec_timeout_explicit_above_default` | `tests/timeout_test.rs` |
| IT-5 | `ec_timeout_unlimited_flag` | `tests/timeout_test.rs` |
| IT-6 | `ec_timeout_unlimited_env` | `tests/timeout_test.rs` |
| IT-7 | `ec_timeout_env_matches_default` | `tests/env_var_test.rs` |
| IT-8 | `ec_timeout_default_kills` | `tests/timeout_test.rs` |

---

### IT-1: `DEFAULT_PRINT_TIMEOUT_SECS` constant equals 3600

- **Given:** source file `src/cli/execution.rs`
- **When:** static source inspection at test run time
- **Then:** File contains (1) `DEFAULT_PRINT_TIMEOUT_SECS : u32 = 3600` — constant exists with correct value; (2) `unwrap_or( DEFAULT_PRINT_TIMEOUT_SECS )` — constant used inside `default_print_timeout()` helper (not inlined at call site); (3) `unwrap_or( default_print_timeout() )` — `run_print_mode()` call site delegates to helper, not the constant directly (TSK-228)
- **Exit:** 0
- **Source:** [invariant/007_print_mode_timeout.md](../../../docs/invariant/007_print_mode_timeout.md) Enforcement Mechanism

---

### IT-2: No `--timeout`, fast subprocess → exit 0, no timeout message

- **Given:** no `--timeout` CLI flag; `CLR_TIMEOUT` removed; fast-exit fake claude; `-p --max-sessions 0`
- **When:** `clr -p --max-sessions 0 "x"` with fast-exit fake; `CLR_TIMEOUT` unset
- **Then:** Exit 0; stderr does NOT contain "timeout"; 3600s watchdog does not trigger for a quickly-exiting subprocess
- **Exit:** 0
- **Source:** [invariant/007_print_mode_timeout.md](../../../docs/invariant/007_print_mode_timeout.md) Invariant Statement

---

### IT-3: No `--timeout`, 2s subprocess → exit 0 in ≤ 10s (3600s watchdog does not fire)

- **Given:** no `--timeout` CLI flag; `CLR_TIMEOUT` removed; fake claude sleeps 2s then exits 0; `-p --max-sessions 0`
- **When:** `clr -p --max-sessions 0 "x"` with 2s-sleep fake; `CLR_TIMEOUT` unset
- **Then:** Exit 0 within ≤ 10s; no "timeout" on stderr; 3600s watchdog is set but the subprocess exits before the deadline
- **Exit:** 0
- **Source:** [invariant/007_print_mode_timeout.md](../../../docs/invariant/007_print_mode_timeout.md) Invariant Statement

---

### IT-4: `--timeout 7200` explicit value wins over 3600 default

- **Given:** `--timeout 7200`; `CLR_TIMEOUT` removed; fast-exit fake claude; `-p --max-sessions 0`
- **When:** `clr -p --timeout 7200 --max-sessions 0 "x"` with fast-exit fake
- **Then:** Exit 0; no "timeout" on stderr; `Some(7200).unwrap_or(3600) = 7200` — explicit wins
- **Exit:** 0
- **Source:** [invariant/007_print_mode_timeout.md](../../../docs/invariant/007_print_mode_timeout.md) "Explicit override still wins"

---

### IT-5: `--timeout 0` opts out of 3600s default (unlimited)

- **Given:** `--timeout 0`; `CLR_TIMEOUT` removed; fast-exit fake claude; `-p --max-sessions 0`
- **When:** `clr -p --timeout 0 --max-sessions 0 "x"` with fast-exit fake
- **Then:** Exit 0; no "timeout" on stderr; `Some(0).unwrap_or(3600) = 0` → unlimited
- **Exit:** 0
- **Source:** [invariant/007_print_mode_timeout.md](../../../docs/invariant/007_print_mode_timeout.md) "Explicit override still wins"

---

### IT-6: `CLR_TIMEOUT=0` opts out of 3600s default (unlimited)

- **Given:** `CLR_TIMEOUT=0`; no `--timeout` CLI flag; fast-exit fake claude; `-p --max-sessions 0`
- **When:** `CLR_TIMEOUT=0 clr -p --max-sessions 0 "x"` with fast-exit fake
- **Then:** Exit 0; no "timeout" on stderr; env var sets `cli.timeout = Some(0)` → `Some(0).unwrap_or(3600) = 0` → unlimited
- **Exit:** 0
- **Source:** [invariant/007_print_mode_timeout.md](../../../docs/invariant/007_print_mode_timeout.md) "Explicit override still wins"

---

### IT-7: `CLR_TIMEOUT=3600` accepted without error

- **Given:** `CLR_TIMEOUT=3600`; `--dry-run "task"`
- **When:** `CLR_TIMEOUT=3600 clr --dry-run "task"`
- **Then:** Exit 0; env var parsed successfully without error; dry-run completes normally
- **Exit:** 0
- **Source:** [invariant/007_print_mode_timeout.md](../../../docs/invariant/007_print_mode_timeout.md), [cli/param/036_timeout.md](../cli/param/036_timeout.md) ec_timeout_env_matches_default

---

### IT-8: Default watchdog fires and kills hanging subprocess

- **Given:** no `--timeout` CLI flag; `CLR_TIMEOUT` removed; `_CLR_DEFAULT_TIMEOUT=2` (overrides `DEFAULT_PRINT_TIMEOUT_SECS` to 2s for testing); fake claude script that sleeps 30s; `-p --max-sessions 0 --retry-override 0`
- **When:** `_CLR_DEFAULT_TIMEOUT=2 clr -p --max-sessions 0 --retry-override 0 "x"` with 30s-sleeping fake; `CLR_TIMEOUT` unset
- **Then:** Exit 4 within ~5s; stderr contains "timeout"; subprocess killed by default watchdog. This proves the `None → unwrap_or(default_print_timeout())` path fires `poll_timeout()` and kills the subprocess — the gap that EC-7 (explicit `--timeout 1`) does not cover.
- **Exit:** 4
- **Source:** [invariant/007_print_mode_timeout.md](../../../docs/invariant/007_print_mode_timeout.md) Invariant Statement, Enforcement Mechanism

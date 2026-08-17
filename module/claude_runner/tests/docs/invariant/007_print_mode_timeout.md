# Test: Invariant — No Built-In Print-Mode Session Timeout

Test case planning for [invariant/007_print_mode_timeout.md](../../../docs/invariant/007_print_mode_timeout.md). Tests validate that no execution path arms a watchdog when `--timeout`/`CLR_TIMEOUT` is unexpressed (`DEFAULT_PRINT_TIMEOUT_SECS = 0`, TSK-503), that expressed values win unchanged, and that the `_CLR_DEFAULT_TIMEOUT` test hook keeps the default-path kill machinery testable.

**Source:** [invariant/007_print_mode_timeout.md](../../../docs/invariant/007_print_mode_timeout.md)
**Related:** [cli/param/036_timeout.md](../cli/param/036_timeout.md), [invariant/006_exit_codes.md](006_exit_codes.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `DEFAULT_PRINT_TIMEOUT_SECS` constant equals `0` in source (TSK-503) | Structural |
| IT-2 | No `--timeout`, fast subprocess → exit 0, no timeout message (nothing armed to fire) | Invariant Hold |
| IT-3 | No `--timeout`, 2s subprocess → exit 0 in ≤ 10s (no default watchdog exists) | Invariant Hold |
| IT-4 | `--timeout 7200` with fast subprocess → expressed large value accepted | Explicit Override |
| IT-5 | `--timeout 0` expresses unlimited explicitly; fast subprocess → exit 0 | Explicit Override |
| IT-6 | `CLR_TIMEOUT=0` expresses unlimited via env; fast subprocess → exit 0 | Env Var Override |
| IT-7 | `CLR_TIMEOUT=3600` accepted without error; dry-run exits 0 | Env Var |
| IT-8 | No `--timeout`; `_CLR_DEFAULT_TIMEOUT=2`; hanging subprocess → exit 4, killed by hook-armed default watchdog | Invariant Kill |

## Test Coverage Summary

- Structural: 1 test (IT-1)
- Invariant Hold: 2 tests (IT-2, IT-3)
- Invariant Kill: 1 test (IT-8)
- Explicit Override: 2 tests (IT-4, IT-5)
- Env Var Override: 1 test (IT-6)
- Env Var: 1 test (IT-7)

**Total:** 8 invariant test cases

## Architectural Constraint

IT-2 and IT-3 use a fake `claude` subprocess to avoid live API calls. IT-3 uses a 2-second sleep script with a ≤10s wall bound — if a nonzero built-in default under 2s were ever reintroduced, the premature kill fails the test; the wall bound catches a hang regression. IT-4 through IT-7 confirm the expressed path (`Some(n)` resolves to `n`). IT-8 uses a test-only internal env var `_CLR_DEFAULT_TIMEOUT` that overrides `DEFAULT_PRINT_TIMEOUT_SECS` (0 in production since TSK-503) to a short value (2s). This keeps the default-path kill machinery — `poll_timeout()`, exit 4, retry, journal emission — verifiable even though production no longer arms it. The env var is prefixed with `_` to signal internal/test-only use and is not documented in user-facing param docs. All 8 tests are fully automated in `timeout_test.rs` and `env_var_test.rs`; no live claude is needed.

## Implementation Notes

| IT | Test Function | File |
|----|---------------|------|
| IT-1 | `ec_timeout_default_constant_value` | `tests/timeout_test.rs` |
| IT-2 | `ec_timeout_default_no_fire` | `tests/timeout_test.rs` |
| IT-3 | `ec_timeout_default_unlimited` | `tests/timeout_test.rs` |
| IT-4 | `ec_timeout_explicit_large_value` | `tests/timeout_test.rs` |
| IT-5 | `ec_timeout_unlimited_flag` | `tests/timeout_test.rs` |
| IT-6 | `ec_timeout_unlimited_env` | `tests/timeout_test.rs` |
| IT-7 | `ec_timeout_env_hour_value_accepted` | `tests/env_var_test.rs` |
| IT-8 | `ec_timeout_default_kills` | `tests/timeout_test.rs` |

---

### IT-1: `DEFAULT_PRINT_TIMEOUT_SECS` constant equals 0

- **Given:** source file `src/cli/execution.rs`
- **When:** static source inspection at test run time
- **Then:** File contains (1) `DEFAULT_PRINT_TIMEOUT_SECS : u32 = 0` — constant exists with the TSK-503 value; (2) `unwrap_or( DEFAULT_PRINT_TIMEOUT_SECS )` — constant used inside `default_print_timeout()` helper (not inlined at call site); (3) `unwrap_or( default_print_timeout() )` — `run_print_mode()` call site delegates to helper, not the constant directly (TSK-228 — this keeps the `_CLR_DEFAULT_TIMEOUT` hook live)
- **Exit:** 0
- **Source:** [invariant/007_print_mode_timeout.md](../../../docs/invariant/007_print_mode_timeout.md) Enforcement Mechanism

---

### IT-2: No `--timeout`, fast subprocess → exit 0, no timeout message

- **Given:** no `--timeout` CLI flag; `CLR_TIMEOUT` removed; fast-exit fake claude; `-p --max-sessions 0`
- **When:** `clr -p --max-sessions 0 "x"` with fast-exit fake; `CLR_TIMEOUT` unset
- **Then:** Exit 0; stderr does NOT contain "timeout"; nothing is armed on the unexpressed path, so no watchdog can trigger
- **Exit:** 0
- **Source:** [invariant/007_print_mode_timeout.md](../../../docs/invariant/007_print_mode_timeout.md) Invariant Statement

---

### IT-3: No `--timeout`, 2s subprocess → exit 0 in ≤ 10s (no default watchdog)

- **Given:** no `--timeout` CLI flag; `CLR_TIMEOUT` removed; fake claude sleeps 2s then exits 0; `-p --max-sessions 0`
- **When:** `clr -p --max-sessions 0 "x"` with 2s-sleep fake; `CLR_TIMEOUT` unset
- **Then:** Exit 0 within ≤ 10s; no "timeout" on stderr; the unexpressed path arms no watchdog (TSK-503) — a reintroduced sub-2s default would kill the subprocess and fail this test
- **Exit:** 0
- **Source:** [invariant/007_print_mode_timeout.md](../../../docs/invariant/007_print_mode_timeout.md) Invariant Statement

---

### IT-4: `--timeout 7200` expressed large value accepted

- **Given:** `--timeout 7200`; `CLR_TIMEOUT` removed; fast-exit fake claude; `-p --max-sessions 0`
- **When:** `clr -p --timeout 7200 --max-sessions 0 "x"` with fast-exit fake
- **Then:** Exit 0; no "timeout" on stderr; `Some(7200)` expressed branch resolves to 7200
- **Exit:** 0
- **Source:** [invariant/007_print_mode_timeout.md](../../../docs/invariant/007_print_mode_timeout.md) "Expressed values win unchanged"

---

### IT-5: `--timeout 0` expresses unlimited explicitly

- **Given:** `--timeout 0`; `CLR_TIMEOUT` removed; fast-exit fake claude; `-p --max-sessions 0`
- **When:** `clr -p --timeout 0 --max-sessions 0 "x"` with fast-exit fake
- **Then:** Exit 0; no "timeout" on stderr; `Some(0)` resolves to 0 → unlimited (same behavior as the TSK-503 default; expressed-zero path pinned)
- **Exit:** 0
- **Source:** [invariant/007_print_mode_timeout.md](../../../docs/invariant/007_print_mode_timeout.md) "Expressed values win unchanged"

---

### IT-6: `CLR_TIMEOUT=0` expresses unlimited via env

- **Given:** `CLR_TIMEOUT=0`; no `--timeout` CLI flag; fast-exit fake claude; `-p --max-sessions 0`
- **When:** `CLR_TIMEOUT=0 clr -p --max-sessions 0 "x"` with fast-exit fake
- **Then:** Exit 0; no "timeout" on stderr; env var sets `cli.timeout = Some(0)` → resolves to 0 → unlimited (same behavior as the TSK-503 default; env-expressed-zero path pinned)
- **Exit:** 0
- **Source:** [invariant/007_print_mode_timeout.md](../../../docs/invariant/007_print_mode_timeout.md) "Expressed values win unchanged"

---

### IT-7: `CLR_TIMEOUT=3600` accepted without error

- **Given:** `CLR_TIMEOUT=3600`; `--dry-run "task"`
- **When:** `CLR_TIMEOUT=3600 clr --dry-run "task"`
- **Then:** Exit 0; env var parsed successfully without error; dry-run completes normally (3600 was the built-in default at authoring time under TSK-227; since TSK-503 it survives as a representative hour-scale expressed value)
- **Exit:** 0
- **Source:** [invariant/007_print_mode_timeout.md](../../../docs/invariant/007_print_mode_timeout.md), [cli/param/036_timeout.md](../cli/param/036_timeout.md) ec_timeout_env_hour_value_accepted

---

### IT-8: Hook-armed default watchdog fires and kills hanging subprocess

- **Given:** no `--timeout` CLI flag; `CLR_TIMEOUT` removed; `_CLR_DEFAULT_TIMEOUT=2` (overrides `DEFAULT_PRINT_TIMEOUT_SECS` — 0 in production since TSK-503 — to 2s for testing); fake claude script that sleeps 30s; `-p --max-sessions 0 --retry-override 0`
- **When:** `_CLR_DEFAULT_TIMEOUT=2 clr -p --max-sessions 0 --retry-override 0 "x"` with 30s-sleeping fake; `CLR_TIMEOUT` unset
- **Then:** Exit 4 within ~5s; stderr contains "timeout"; subprocess killed by the hook-armed default watchdog. This proves the `None → unwrap_or(default_print_timeout())` path still fires `poll_timeout()` when the hook arms it — the machinery production no longer triggers by default, and the gap that EC-7 (explicit `--timeout 1`) does not cover.
- **Exit:** 4
- **Source:** [invariant/007_print_mode_timeout.md](../../../docs/invariant/007_print_mode_timeout.md) Invariant Statement, Enforcement Mechanism

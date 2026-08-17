# Parameter :: `--timeout` (run/ask)

Edge case coverage for the `--timeout` parameter on the `run`/`ask` dispatch paths. See [036_timeout.md](../../../../docs/cli/param/036_timeout.md) for specification.

**Scope note:** This file covers `--timeout` for the `run`/`ask` commands only. `--timeout` for
the `isolated`/`refresh` commands is covered in [020_timeout.md](020_timeout.md). All four commands
now share the same semantics: `--timeout 0` means **unlimited** (no watchdog). Tests in this file
must not be confused with those in `020_timeout.md`.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--timeout` (run/ask help) | Documentation |
| EC-2 | `--timeout 0 --dry-run` → exit 0; explicit unlimited (matches the TSK-503 default; self-documenting opt-out) | Behavioral Divergence |
| EC-3 | `--timeout 30 --dry-run` → exit 0; 30s watchdog accepted | Behavioral Divergence |
| EC-4 | `CLR_TIMEOUT=10 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_TIMEOUT=5 --timeout 60 --dry-run` → CLI 60 wins | CLI-wins |
| EC-6 | `CLR_TIMEOUT=abc --dry-run` → silently ignored; default `DEFAULT_PRINT_TIMEOUT_SECS` (0 = unlimited) for print-mode | Validation |
| EC-7 | Fake script sleeps 30; `--timeout 1` → exit 4 within ~2s; stderr contains "timeout" | Integration |
| EC-8 | Fake script exits 0 quickly; `--timeout 30` → exit 0; no timeout message | Integration |
| ec_timeout_default_constant_value | `DEFAULT_PRINT_TIMEOUT_SECS` constant equals `0` in source (TSK-503) | Structural |
| ec_timeout_default_no_fire | No `--timeout`, no `CLR_TIMEOUT`; fast subprocess → exit 0, no timeout msg | Integration |
| ec_timeout_default_unlimited | No `--timeout`, no `CLR_TIMEOUT`; 2s subprocess → exit 0 in ≤10s (no default watchdog) | Integration |
| ec_timeout_explicit_large_value | `--timeout 7200` with fast subprocess → exit 0, no timeout msg | Integration |
| ec_timeout_unlimited_flag | `--timeout 0` expresses unlimited explicitly; fast subprocess → exit 0 | Integration |
| ec_timeout_unlimited_env | `CLR_TIMEOUT=0` expresses unlimited via env; fast subprocess → exit 0 | Env Var |
| ec_timeout_env_hour_value_accepted | `CLR_TIMEOUT=3600` accepted without error; dry-run exits 0 | Env Var |
| ec_timeout_default_kills | No `--timeout`, `_CLR_DEFAULT_TIMEOUT=2`; hanging subprocess → exit 4, killed by hook-armed default watchdog | Integration (TSK-227) |
| ec_timeout_retry_no_double_emission | `_CLR_DEFAULT_TIMEOUT=2`, `--retry-on-process 1`, `--process-delay 0`; hanging subprocess → no stderr line starts with `"timeout after"` | Bug Reproducer (BUG-317) |

## Test Coverage Summary

- Documentation: 1 test (EC-1)
- Behavioral Divergence: 2 tests (EC-2, EC-3)
- Env Var: 3 tests (EC-4, ec_timeout_unlimited_env, ec_timeout_env_hour_value_accepted)
- CLI-wins: 1 test (EC-5)
- Validation: 1 test (EC-6)
- Integration: 2 tests (EC-7, EC-8)
- Structural: 1 test (ec_timeout_default_constant_value)
- Integration (TSK-227): 5 tests (ec_timeout_default_no_fire, ec_timeout_default_unlimited, ec_timeout_explicit_large_value, ec_timeout_unlimited_flag, ec_timeout_default_kills)
- Bug Reproducer (BUG-317): 1 test (ec_timeout_retry_no_double_emission)

**Total:** 17 edge cases

## Architectural Constraint

The watchdog behavior (SIGKILL after N seconds) requires a live subprocess. EC-7 and EC-8 use
a fake claude script injected via PATH override (same pattern as `output_file_test.rs` and
`expect_validation_test.rs`). EC-7 is the primary behavioral integration test: the fake script
sleeps 30 seconds but the timeout fires after 1 second, producing exit 4 and a stderr message
containing "timeout". EC-8 verifies the no-timeout path: the fake script exits immediately and
the timeout watchdog is disarmed without firing.

**Cross-command parity with 020_timeout.md:** All four commands now use the same `--timeout 0`
semantics: unlimited (no watchdog). Tests in this file cover `run`/`ask` only;
`isolated`/`refresh` timeout tests are in `020_timeout.md`.

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_timeout_help_listed` | `timeout_test.rs` |
| EC-2 | `ec2_timeout_zero_dry_run` | `timeout_test.rs` |
| EC-3 | `ec3_timeout_nonzero_dry_run` | `timeout_test.rs` |
| EC-4 | `ec4_clr_timeout_env_var_accepted` | `timeout_test.rs` |
| EC-5 | `ec5_timeout_cli_wins_over_env` | `timeout_test.rs` |
| EC-6 | `ec6_clr_timeout_invalid_ignored` | `timeout_test.rs` |
| EC-7 | `ec7_timeout_fires_kills_subprocess` | `timeout_test.rs` |
| EC-8 | `ec8_no_timeout_when_subprocess_exits_fast` | `timeout_test.rs` |
| ec_timeout_default_constant_value | `ec_timeout_default_constant_value` | `timeout_test.rs` |
| ec_timeout_default_no_fire | `ec_timeout_default_no_fire` | `timeout_test.rs` |
| ec_timeout_default_unlimited | `ec_timeout_default_unlimited` | `timeout_test.rs` |
| ec_timeout_explicit_large_value | `ec_timeout_explicit_large_value` | `timeout_test.rs` |
| ec_timeout_unlimited_flag | `ec_timeout_unlimited_flag` | `timeout_test.rs` |
| ec_timeout_unlimited_env | `ec_timeout_unlimited_env` | `timeout_test.rs` |
| ec_timeout_env_hour_value_accepted | `ec_timeout_env_hour_value_accepted` | `env_var_test.rs` |
| ec_timeout_default_kills | `ec_timeout_default_kills` | `timeout_test.rs` |
| ec_timeout_retry_no_double_emission | `ec_timeout_retry_no_double_emission` | `timeout_test.rs` |

---

### EC-1: --help (run/ask) lists --timeout

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--timeout`
- **Exit:** 0
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md)
- **Commands:** run, ask

---

### EC-2: --timeout 0 --dry-run → exit 0; explicit unlimited

- **Given:** `--timeout 0` and `--dry-run` set
- **When:** `clr --timeout 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced; `Some(0).unwrap_or(DEFAULT_PRINT_TIMEOUT_SECS) = 0` → no watchdog. **Divergence from EC-3:** value 0 disables the watchdog entirely — no `child.kill()` thread is spawned; value 30 (EC-3) activates the watchdog with a 30-second countdown. Since TSK-503 explicit `--timeout 0` matches the built-in default; it remains the self-documenting opt-out (and contributes no gate-wait budget — BUG-445)
- **Exit:** 0
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md)
- **Commands:** run, ask

---

### EC-3: --timeout 30 --dry-run → exit 0; 30s watchdog accepted

- **Given:** `--timeout 30` and `--dry-run` set
- **When:** `clr --timeout 30 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced; flag accepted without error (no subprocess spawned in dry-run so watchdog is never started)
- **Exit:** 0
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md)
- **Commands:** run, ask

---

### EC-4: CLR_TIMEOUT=10 env var → applied when CLI flag absent

- **Given:** `CLR_TIMEOUT=10` set; no `--timeout` CLI flag; `--dry-run` set
- **When:** `CLR_TIMEOUT=10 clr --dry-run "task"`
- **Then:** Exit 0; env var accepted; dry-run output produced (watchdog skipped in dry-run)
- **Exit:** 0
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md)
- **Commands:** run, ask

---

### EC-5: --timeout CLI wins over CLR_TIMEOUT env var

- **Given:** `CLR_TIMEOUT=5` set; `--timeout 60` on CLI; `--dry-run` set
- **When:** `CLR_TIMEOUT=5 clr --timeout 60 --dry-run "task"`
- **Then:** Exit 0; CLI value 60 used (env var 5 ignored); dry-run output produced
- **Exit:** 0
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md)
- **Commands:** run, ask

---

### EC-6: CLR_TIMEOUT=invalid → silently ignored; default DEFAULT_PRINT_TIMEOUT_SECS (0 = unlimited) used

- **Given:** `CLR_TIMEOUT=abc` set; no `--timeout` CLI flag; `--dry-run` set
- **When:** `CLR_TIMEOUT=abc clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored; `cli.timeout` stays at `None`; `None.unwrap_or(DEFAULT_PRINT_TIMEOUT_SECS) = 0` for print-mode (unlimited since TSK-503)
- **Exit:** 0
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md)
- **Commands:** run, ask

---

### EC-7: Timeout fires → exit 4; stderr contains "timeout"

- **Given:** fake claude script that sleeps 30 seconds; `--timeout 1 -p "x"`
- **When:** `clr --timeout 1 -p "x"` using fake sleeping script
- **Then:** Exit 4 within ~2 seconds (watchdog kills subprocess after 1s); stderr contains "timeout after 1s" (or equivalent message); no stdout emitted
- **Exit:** 4
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md)
- **Commands:** run, ask

---

### EC-8: No timeout fires when subprocess exits before deadline

- **Given:** fake claude script that exits 0 immediately (emits nothing, exits fast); `--timeout 30 -p "x"`
- **When:** `clr --timeout 30 -p "x"` using fast-exit fake script
- **Then:** Exit 0; no "timeout" message on stderr; subprocess completes normally before watchdog fires; watchdog thread disarmed
- **Exit:** 0
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md)
- **Commands:** run, ask

---

### ec_timeout_default_constant_value: DEFAULT_PRINT_TIMEOUT_SECS equals 0

- **Given:** source file `src/cli/execution.rs` at build time
- **When:** `include_str!("../src/cli/execution.rs")` — static assertion at test compile/run time
- **Then:** File contains (1) `DEFAULT_PRINT_TIMEOUT_SECS : u32 = 0` (TSK-503 — no built-in watchdog); (2) `unwrap_or( DEFAULT_PRINT_TIMEOUT_SECS )` inside the `default_print_timeout()` helper; (3) `unwrap_or( default_print_timeout() )` at the `run_print_mode()` call site (TSK-228 — not the constant directly, so the `_CLR_DEFAULT_TIMEOUT` hook stays live)
- **Exit:** 0
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md), [invariant/007_print_mode_timeout.md](../../../../docs/invariant/007_print_mode_timeout.md)
- **Commands:** run, ask

---

### ec_timeout_default_no_fire: no --timeout, fast subprocess → exit 0 (BUG-305)

- **Given:** no `--timeout` CLI flag; `CLR_TIMEOUT` removed from env; fake claude script that exits 0 immediately; `-p "x" --max-sessions 0`
- **When:** `clr -p --max-sessions 0 "x"` using fast-exit fake script, `CLR_TIMEOUT` unset
- **Then:** Exit 0; stderr does NOT contain "timeout"; fast subprocess completes on the unexpressed path without incident (no default watchdog exists to fire — TSK-503)
- **Exit:** 0
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md), [invariant/007_print_mode_timeout.md](../../../../docs/invariant/007_print_mode_timeout.md)
- **Commands:** run, ask

---

### ec_timeout_default_unlimited: 2s subprocess completes, no default watchdog armed

- **Given:** no `--timeout` CLI flag; `CLR_TIMEOUT` removed; fake claude sleeps 2s then exits 0; `-p --max-sessions 0`
- **When:** `clr -p --max-sessions 0 "x"` using 2s-sleep fake, `CLR_TIMEOUT` unset
- **Then:** Exit 0 within ≤10s; no "timeout" on stderr; the unexpressed path arms nothing (TSK-503). If a nonzero built-in default under 2 s were ever reintroduced, the subprocess would be killed prematurely and this test would fail
- **Exit:** 0
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md), [invariant/007_print_mode_timeout.md](../../../../docs/invariant/007_print_mode_timeout.md)
- **Commands:** run, ask

---

### ec_timeout_explicit_large_value: --timeout 7200 with fast subprocess

- **Given:** `--timeout 7200` CLI flag; fast-exit fake claude; `-p --max-sessions 0 "x"`
- **When:** `clr -p --timeout 7200 --max-sessions 0 "x"` using fast-exit fake
- **Then:** Exit 0; no "timeout" on stderr; `Some(7200)` expressed branch resolves to 7200 (explicit wins)
- **Exit:** 0
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md)
- **Commands:** run, ask

---

### ec_timeout_unlimited_flag: --timeout 0 expresses unlimited explicitly

- **Given:** `--timeout 0` CLI flag; `CLR_TIMEOUT` removed; fast-exit fake claude; `-p --max-sessions 0 "x"`
- **When:** `clr -p --timeout 0 --max-sessions 0 "x"` using fast-exit fake, `CLR_TIMEOUT` unset
- **Then:** Exit 0; no "timeout" on stderr; `Some(0)` resolves to 0 → unlimited (same as the TSK-503 default; the expressed-zero path stays intact)
- **Exit:** 0
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md)
- **Commands:** run, ask

---

### ec_timeout_unlimited_env: CLR_TIMEOUT=0 expresses unlimited via env

- **Given:** `CLR_TIMEOUT=0`; no `--timeout` CLI flag; fast-exit fake claude; `-p --max-sessions 0 "x"`
- **When:** `CLR_TIMEOUT=0 clr -p --max-sessions 0 "x"` using fast-exit fake
- **Then:** Exit 0; no "timeout" on stderr; env var sets `cli.timeout = Some(0)` → resolves to 0 → unlimited (same as the TSK-503 default; env-expressed-zero path stays intact)
- **Exit:** 0
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md)
- **Commands:** run, ask

---

### ec_timeout_env_hour_value_accepted: CLR_TIMEOUT=3600 is valid and accepted

- **Given:** `CLR_TIMEOUT=3600`; `--dry-run "task"`
- **When:** `CLR_TIMEOUT=3600 clr --dry-run "task"`
- **Then:** Exit 0; env var parsed successfully without error; dry-run completes normally. (3600 was the built-in default when this test was written under TSK-227; since TSK-503 it survives purely as a representative hour-scale expressed value)
- **Exit:** 0
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md)
- **Commands:** run, ask

---

### ec_timeout_default_kills: default watchdog fires and kills hanging subprocess

- **Given:** no `--timeout` CLI flag; `CLR_TIMEOUT` removed; `_CLR_DEFAULT_TIMEOUT=2`; fake claude sleeps 30s; `-p --max-sessions 0 --retry-override 0 "x"`
- **When:** `_CLR_DEFAULT_TIMEOUT=2 clr -p --max-sessions 0 --retry-override 0 "x"` with 30s-sleeping fake; `CLR_TIMEOUT` unset
- **Then:** Exit 4 within ~5s; stderr contains "timeout"; subprocess killed by the hook-armed default watchdog (`_CLR_DEFAULT_TIMEOUT=2` — the production default is 0/unlimited since TSK-503). Proves the `None → unwrap_or(default_print_timeout())` path fires `poll_timeout()`. EC-7 tests `Some(1)` (explicit `--timeout`); this test covers the `None` (no flag) path.
- **Exit:** 4
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md), [invariant/007_print_mode_timeout.md](../../../../docs/invariant/007_print_mode_timeout.md)
- **Commands:** run, ask

---

### ec_timeout_retry_no_double_emission: BUG-317 — [Process] retry line starts at column 0

- **Given:** `_CLR_DEFAULT_TIMEOUT=2`; `CLR_TIMEOUT` removed; `--retry-on-process 1 --process-delay 0 --max-sessions 0 -p "x"`; fake claude sleeps 300s
- **When:** `_CLR_DEFAULT_TIMEOUT=2 clr -p --retry-on-process 1 --process-delay 0 --max-sessions 0 "x"` with indefinitely-sleeping fake; `CLR_TIMEOUT` unset
- **Then:** No stderr line begins with `"timeout after"`; at least one `[Process]` line is present in stderr. Pre-fix: `"timeout after 2s[Process] timeout after 2s — retrying…"` on one line. Post-fix: `"[Process] timeout after 2s — retrying…"` cleanly at column 0.
- **Exit:** 4 (timeout exhausted after two attempts: 1 retry = 2 total; both timeout)
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md)
- **Commands:** run, ask

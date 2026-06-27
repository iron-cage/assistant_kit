# Parameter :: `--timeout` (run/ask)

Edge case coverage for the `--timeout` parameter on the `run`/`ask` dispatch paths. See [036_timeout.md](../../../../docs/cli/param/036_timeout.md) for specification.

**Scope note:** This file covers `--timeout` for the `run`/`ask` commands only. `--timeout` for
the `isolated`/`refresh` commands is covered in [20_timeout.md](20_timeout.md). All four commands
now share the same semantics: `--timeout 0` means **unlimited** (no watchdog). Tests in this file
must not be confused with those in `20_timeout.md`.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--timeout` (run/ask help) | Documentation |
| EC-2 | `--timeout 0 --dry-run` → exit 0; explicit unlimited (overrides 3600s print-mode default) | Behavioral Divergence |
| EC-3 | `--timeout 30 --dry-run` → exit 0; 30s watchdog accepted | Behavioral Divergence |
| EC-4 | `CLR_TIMEOUT=10 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_TIMEOUT=5 --timeout 60 --dry-run` → CLI 60 wins | CLI-wins |
| EC-6 | `CLR_TIMEOUT=abc --dry-run` → silently ignored; default `DEFAULT_PRINT_TIMEOUT_SECS` (3600) for print-mode | Validation |
| EC-7 | Fake script sleeps 30; `--timeout 1` → exit 4 within ~2s; stderr contains "timeout" | Integration |
| EC-8 | Fake script exits 0 quickly; `--timeout 30` → exit 0; no timeout message | Integration |
| ec_timeout_default_constant_value | `DEFAULT_PRINT_TIMEOUT_SECS` constant equals `3600` in source | Structural |
| ec_timeout_default_no_fire | No `--timeout`, no `CLR_TIMEOUT`; fast subprocess → exit 0, no timeout msg | Integration |
| ec_timeout_default_activates_watchdog | No `--timeout`, no `CLR_TIMEOUT`; 2s subprocess → exit 0 in ≤10s (3600s watchdog) | Integration |
| ec_timeout_explicit_above_default | `--timeout 7200` with fast subprocess → exit 0, no timeout msg | Integration |
| ec_timeout_unlimited_flag | `--timeout 0` opts out of 3600s default; fast subprocess → exit 0 | Integration |
| ec_timeout_unlimited_env | `CLR_TIMEOUT=0` opts out of 3600s default; fast subprocess → exit 0 | Env Var |
| ec_timeout_env_matches_default | `CLR_TIMEOUT=3600` accepted without error; dry-run exits 0 | Env Var |
| ec_timeout_default_kills | No `--timeout`, `_CLR_DEFAULT_TIMEOUT=2`; hanging subprocess → exit 4, killed by default watchdog | Integration (TSK-227) |
| ec_timeout_retry_no_double_emission | `_CLR_DEFAULT_TIMEOUT=2`, `--retry-on-process 1`, `--process-delay 0`; hanging subprocess → no stderr line starts with `"timeout after"` | Bug Reproducer (BUG-317) |

## Test Coverage Summary

- Documentation: 1 test (EC-1)
- Behavioral Divergence: 2 tests (EC-2, EC-3)
- Env Var: 3 tests (EC-4, ec_timeout_unlimited_env, ec_timeout_env_matches_default)
- CLI-wins: 1 test (EC-5)
- Validation: 1 test (EC-6)
- Integration: 2 tests (EC-7, EC-8)
- Structural: 1 test (ec_timeout_default_constant_value)
- Integration (TSK-227): 5 tests (ec_timeout_default_no_fire, ec_timeout_default_activates_watchdog, ec_timeout_explicit_above_default, ec_timeout_unlimited_flag, ec_timeout_default_kills)
- Bug Reproducer (BUG-317): 1 test (ec_timeout_retry_no_double_emission)

**Total:** 17 edge cases

## Architectural Constraint

The watchdog behavior (SIGKILL after N seconds) requires a live subprocess. EC-7 and EC-8 use
a fake claude script injected via PATH override (same pattern as `output_file_test.rs` and
`expect_validation_test.rs`). EC-7 is the primary behavioral integration test: the fake script
sleeps 30 seconds but the timeout fires after 1 second, producing exit 4 and a stderr message
containing "timeout". EC-8 verifies the no-timeout path: the fake script exits immediately and
the timeout watchdog is disarmed without firing.

**Cross-command parity with 20_timeout.md:** All four commands now use the same `--timeout 0`
semantics: unlimited (no watchdog). Tests in this file cover `run`/`ask` only;
`isolated`/`refresh` timeout tests are in `20_timeout.md`.

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
| ec_timeout_default_activates_watchdog | `ec_timeout_default_activates_watchdog` | `timeout_test.rs` |
| ec_timeout_explicit_above_default | `ec_timeout_explicit_above_default` | `timeout_test.rs` |
| ec_timeout_unlimited_flag | `ec_timeout_unlimited_flag` | `timeout_test.rs` |
| ec_timeout_unlimited_env | `ec_timeout_unlimited_env` | `timeout_test.rs` |
| ec_timeout_env_matches_default | `ec_timeout_env_matches_default` | `env_var_test.rs` |
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
- **Then:** Exit 0; dry-run output produced; `Some(0).unwrap_or(DEFAULT_PRINT_TIMEOUT_SECS) = 0` → no watchdog. **Divergence from EC-3:** value 0 disables the watchdog entirely — no `child.kill()` thread is spawned; value 30 (EC-3) activates the watchdog with a 30-second countdown. Explicit `--timeout 0` opts out of the 3600s print-mode default (TSK-227)
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

### EC-6: CLR_TIMEOUT=invalid → silently ignored; default DEFAULT_PRINT_TIMEOUT_SECS (3600) used

- **Given:** `CLR_TIMEOUT=abc` set; no `--timeout` CLI flag; `--dry-run` set
- **When:** `CLR_TIMEOUT=abc clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored; `cli.timeout` stays at `None`; `None.unwrap_or(DEFAULT_PRINT_TIMEOUT_SECS) = 3600` for print-mode
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

### ec_timeout_default_constant_value: DEFAULT_PRINT_TIMEOUT_SECS equals 3600

- **Given:** source file `src/cli/execution.rs` at build time
- **When:** `include_str!("../src/cli/execution.rs")` — static assertion at test compile/run time
- **Then:** File contains (1) `DEFAULT_PRINT_TIMEOUT_SECS : u32 = 3600`; (2) `unwrap_or( DEFAULT_PRINT_TIMEOUT_SECS )` inside the `default_print_timeout()` helper; (3) `unwrap_or( default_print_timeout() )` at the `run_print_mode()` call site (TSK-228 — not the constant directly)
- **Exit:** 0
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md), [invariant/007_print_mode_timeout.md](../../../../docs/invariant/007_print_mode_timeout.md)
- **Commands:** run, ask

---

### ec_timeout_default_no_fire: no --timeout, fast subprocess → exit 0 (BUG-305)

- **Given:** no `--timeout` CLI flag; `CLR_TIMEOUT` removed from env; fake claude script that exits 0 immediately; `-p "x" --max-sessions 0`
- **When:** `clr -p --max-sessions 0 "x"` using fast-exit fake script, `CLR_TIMEOUT` unset
- **Then:** Exit 0; stderr does NOT contain "timeout"; fast subprocess completes under 3600s default watchdog without incident
- **Exit:** 0
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md), [invariant/007_print_mode_timeout.md](../../../../docs/invariant/007_print_mode_timeout.md)
- **Commands:** run, ask

---

### ec_timeout_default_activates_watchdog: 2s subprocess survives 3600s default

- **Given:** no `--timeout` CLI flag; `CLR_TIMEOUT` removed; fake claude sleeps 2s then exits 0; `-p --max-sessions 0`
- **When:** `clr -p --max-sessions 0 "x"` using 2s-sleep fake, `CLR_TIMEOUT` unset
- **Then:** Exit 0 within ≤10s; no "timeout" on stderr; 3600s watchdog does not fire before 2s subprocess completes. If `DEFAULT_PRINT_TIMEOUT_SECS` were < 2, this test would fail — proving the constant is ≥ 2
- **Exit:** 0
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md), [invariant/007_print_mode_timeout.md](../../../../docs/invariant/007_print_mode_timeout.md)
- **Commands:** run, ask

---

### ec_timeout_explicit_above_default: --timeout 7200 with fast subprocess

- **Given:** `--timeout 7200` CLI flag; fast-exit fake claude; `-p --max-sessions 0 "x"`
- **When:** `clr -p --timeout 7200 --max-sessions 0 "x"` using fast-exit fake
- **Then:** Exit 0; no "timeout" on stderr; `Some(7200).unwrap_or(3600) = 7200` (explicit wins)
- **Exit:** 0
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md)
- **Commands:** run, ask

---

### ec_timeout_unlimited_flag: --timeout 0 opts out of 3600s print-mode default

- **Given:** `--timeout 0` CLI flag; `CLR_TIMEOUT` removed; fast-exit fake claude; `-p --max-sessions 0 "x"`
- **When:** `clr -p --timeout 0 --max-sessions 0 "x"` using fast-exit fake, `CLR_TIMEOUT` unset
- **Then:** Exit 0; no "timeout" on stderr; `Some(0).unwrap_or(3600) = 0` → unlimited
- **Exit:** 0
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md)
- **Commands:** run, ask

---

### ec_timeout_unlimited_env: CLR_TIMEOUT=0 opts out of 3600s print-mode default

- **Given:** `CLR_TIMEOUT=0`; no `--timeout` CLI flag; fast-exit fake claude; `-p --max-sessions 0 "x"`
- **When:** `CLR_TIMEOUT=0 clr -p --max-sessions 0 "x"` using fast-exit fake
- **Then:** Exit 0; no "timeout" on stderr; env var sets `cli.timeout = Some(0)` → `Some(0).unwrap_or(3600) = 0` → unlimited
- **Exit:** 0
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md)
- **Commands:** run, ask

---

### ec_timeout_env_matches_default: CLR_TIMEOUT=3600 is valid and accepted

- **Given:** `CLR_TIMEOUT=3600`; `--dry-run "task"`
- **When:** `CLR_TIMEOUT=3600 clr --dry-run "task"`
- **Then:** Exit 0; env var parsed successfully without error; dry-run completes normally
- **Exit:** 0
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md)
- **Commands:** run, ask

---

### ec_timeout_default_kills: default watchdog fires and kills hanging subprocess

- **Given:** no `--timeout` CLI flag; `CLR_TIMEOUT` removed; `_CLR_DEFAULT_TIMEOUT=2`; fake claude sleeps 30s; `-p --max-sessions 0 --retry-override 0 "x"`
- **When:** `_CLR_DEFAULT_TIMEOUT=2 clr -p --max-sessions 0 --retry-override 0 "x"` with 30s-sleeping fake; `CLR_TIMEOUT` unset
- **Then:** Exit 4 within ~5s; stderr contains "timeout"; subprocess killed by default watchdog. Proves the `None → unwrap_or(default_print_timeout())` path fires `poll_timeout()`. EC-7 tests `Some(1)` (explicit `--timeout`); this test covers the `None` (no flag) path.
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

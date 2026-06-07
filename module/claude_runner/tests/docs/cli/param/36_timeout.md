# Parameter :: `--timeout` (run/ask)

Edge case coverage for the `--timeout` parameter on the `run`/`ask` dispatch paths. See [036_timeout.md](../../../../docs/cli/param/036_timeout.md) for specification.

**Scope note:** This file covers `--timeout` for the `run`/`ask` commands only. `--timeout` for
the `isolated`/`refresh` commands is covered in [20_timeout.md](20_timeout.md). The semantics
differ: for `run`/`ask`, `--timeout 0` means **unlimited** (current default behavior preserved);
for `isolated`/`refresh`, `--timeout 0` means **immediate expiry**. Tests in this file must not
be confused with those in `20_timeout.md`.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--timeout` (run/ask help) | Documentation |
| EC-2 | `--timeout 0 --dry-run` → exit 0; unlimited mode (default) | Behavioral Divergence |
| EC-3 | `--timeout 30 --dry-run` → exit 0; 30s watchdog accepted | Behavioral Divergence |
| EC-4 | `CLR_TIMEOUT=10 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_TIMEOUT=5 --timeout 60 --dry-run` → CLI 60 wins | CLI-wins |
| EC-6 | `CLR_TIMEOUT=abc --dry-run` → silently ignored; default 0 (unlimited) | Validation |
| EC-7 | Fake script sleeps 30; `--timeout 1` → exit 2 within ~2s; stderr contains "timeout" | Integration |
| EC-8 | Fake script exits 0 quickly; `--timeout 30` → exit 0; no timeout message | Integration |

## Test Coverage Summary

- Documentation: 1 test (EC-1)
- Behavioral Divergence: 2 tests (EC-2, EC-3)
- Env Var: 1 test (EC-4)
- CLI-wins: 1 test (EC-5)
- Validation: 1 test (EC-6)
- Integration: 2 tests (EC-7, EC-8)

**Total:** 8 edge cases

## Architectural Constraint

The watchdog behavior (SIGKILL after N seconds) requires a live subprocess. EC-7 and EC-8 use
a fake claude script injected via PATH override (same pattern as `output_file_test.rs` and
`expect_validation_test.rs`). EC-7 is the primary behavioral integration test: the fake script
sleeps 30 seconds but the timeout fires after 1 second, producing exit 2 and a stderr message
containing "timeout". EC-8 verifies the no-timeout path: the fake script exits immediately and
the timeout watchdog is disarmed without firing.

**Semantic distinction from 20_timeout.md:** `--timeout 0` for `run`/`ask` means unlimited
(watchdog not started). `--timeout 0` for `isolated`/`refresh` means immediate expiry (the
timeout deadline is set to `now`, so the subprocess is killed before it can produce output).
Tests in this file must never invoke `clr isolated` or `clr refresh`.

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

---

### EC-1: --help (run/ask) lists --timeout

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--timeout`
- **Exit:** 0
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md)
- **Commands:** run, ask

---

### EC-2: --timeout 0 --dry-run → exit 0; unlimited mode

- **Given:** `--timeout 0` and `--dry-run` set
- **When:** `clr --timeout 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced; no watchdog started (0 = unlimited, not immediate expiry). **Divergence from EC-3:** value 0 disables the watchdog entirely — no `child.kill()` thread is spawned; value 30 (EC-3) activates the watchdog code path with a 30-second countdown. **Semantic contrast with isolated:** in `clr isolated`, `--timeout 0` fires immediately (deadline = now); here it preserves current unlimited behavior
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

### EC-6: CLR_TIMEOUT=invalid → silently ignored; default 0 (unlimited)

- **Given:** `CLR_TIMEOUT=abc` set; no `--timeout` CLI flag; `--dry-run` set
- **When:** `CLR_TIMEOUT=abc clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored; default 0 used (unlimited, no watchdog)
- **Exit:** 0
- **Source:** [036_timeout.md](../../../../docs/cli/param/036_timeout.md)
- **Commands:** run, ask

---

### EC-7: Timeout fires → exit 2; stderr contains "timeout"

- **Given:** fake claude script that sleeps 30 seconds; `--timeout 1 -p "x"`
- **When:** `clr --timeout 1 -p "x"` using fake sleeping script
- **Then:** Exit 2 within ~2 seconds (watchdog kills subprocess after 1s); stderr contains "timeout after 1s" (or equivalent message); no stdout emitted
- **Exit:** 2
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

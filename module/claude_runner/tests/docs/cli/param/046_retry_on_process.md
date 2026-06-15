# Parameter :: `--retry-on-process`

Edge case coverage for the `--retry-on-process` parameter (new in retry system redesign).
See [046_retry_on_process.md](../../../../docs/cli/param/046_retry_on_process.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--retry-on-process` | Documentation |
| EC-2 | `--retry-on-process 0 --dry-run` → exit 0; explicit zero accepted | Behavioral Divergence |
| EC-3 | `--retry-on-process 2 --dry-run` → exit 0; nonzero accepted | Behavioral Divergence |
| EC-4 | `CLR_RETRY_ON_PROCESS=2 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_RETRY_ON_PROCESS=1 --retry-on-process 3 --dry-run` → CLI 3 wins | CLI-wins |
| EC-6 | `CLR_RETRY_ON_PROCESS=notanumber --dry-run` → silently ignored | Validation |
| EC-7 | Fake exits 4 once then 0; retries=1, delay=0 → exit 0; `[Process]` in stderr | Integration |
| EC-8 | Fake always exits 4; retries=2 → exit 4; `[Process]` exhaustion in stderr | Integration |

## Test Coverage Summary

- Documentation: 1 test (EC-1)
- Behavioral Divergence: 2 tests (EC-2, EC-3)
- Env Var: 1 test (EC-4)
- CLI-wins: 1 test (EC-5)
- Validation: 1 test (EC-6)
- Integration: 2 tests (EC-7, EC-8)

**Total:** 8 edge cases

## Architectural Constraint

Process class is triggered when the subprocess exits with code 4 (watchdog timeout path).
A fake script that unconditionally exits 4 classifies as Process. `--process-delay 0` is
required in integration tests. No text pattern recognition needed — exit code 4 alone
determines Process classification.

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_retry_on_process_help_listed` | `retry_process_test.rs` |
| EC-2 | `ec2_retry_on_process_zero_dry_run` | `retry_process_test.rs` |
| EC-3 | `ec3_retry_on_process_nonzero_dry_run` | `retry_process_test.rs` |
| EC-4 | `ec4_clr_retry_on_process_env_var_accepted` | `retry_process_test.rs` |
| EC-5 | `ec5_retry_on_process_cli_wins_over_env` | `retry_process_test.rs` |
| EC-6 | `ec6_clr_retry_on_process_invalid_ignored` | `retry_process_test.rs` |
| EC-7 | `ec7_process_retry_succeeds_after_one_exit4` | `retry_process_test.rs` |
| EC-8 | `ec8_process_retry_exhausted` | `retry_process_test.rs` |

---

### EC-1: --help lists --retry-on-process

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--retry-on-process`
- **Exit:** 0
- **Source:** [046_retry_on_process.md](../../../../docs/cli/param/046_retry_on_process.md)
- **Commands:** run, ask

---

### EC-2: --retry-on-process 0 --dry-run → exit 0; explicit zero accepted

- **Given:** `--retry-on-process 0` and `--dry-run` set
- **When:** `clr --retry-on-process 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced; no retry messages. **Divergence from EC-3:** 0 disables Process retry; 2 enables it
- **Exit:** 0
- **Source:** [046_retry_on_process.md](../../../../docs/cli/param/046_retry_on_process.md)
- **Commands:** run, ask

---

### EC-3: --retry-on-process 2 --dry-run → exit 0; nonzero accepted

- **Given:** `--retry-on-process 2` and `--dry-run` set
- **When:** `clr --retry-on-process 2 --dry-run "task"`
- **Then:** Exit 0; flag accepted without error
- **Exit:** 0
- **Source:** [046_retry_on_process.md](../../../../docs/cli/param/046_retry_on_process.md)
- **Commands:** run, ask

---

### EC-4: CLR_RETRY_ON_PROCESS=2 env var → applied when CLI flag absent

- **Given:** `CLR_RETRY_ON_PROCESS=2` set; no CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_ON_PROCESS=2 clr --dry-run "task"`
- **Then:** Exit 0; env var accepted
- **Exit:** 0
- **Source:** [046_retry_on_process.md](../../../../docs/cli/param/046_retry_on_process.md)
- **Commands:** run, ask

---

### EC-5: CLI wins over CLR_RETRY_ON_PROCESS

- **Given:** `CLR_RETRY_ON_PROCESS=1` set; `--retry-on-process 3` on CLI; `--dry-run` set
- **When:** `CLR_RETRY_ON_PROCESS=1 clr --retry-on-process 3 --dry-run "task"`
- **Then:** Exit 0; CLI value 3 used
- **Exit:** 0
- **Source:** [046_retry_on_process.md](../../../../docs/cli/param/046_retry_on_process.md)
- **Commands:** run, ask

---

### EC-6: CLR_RETRY_ON_PROCESS=invalid → silently ignored

- **Given:** `CLR_RETRY_ON_PROCESS=notanumber` set; no CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_ON_PROCESS=notanumber clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored
- **Exit:** 0
- **Source:** [046_retry_on_process.md](../../../../docs/cli/param/046_retry_on_process.md)
- **Commands:** run, ask

---

### EC-7: Process retry succeeds after one exit-4 failure

- **Given:** fake exits 4 on first call; exits 0 on second; `--retry-on-process 1 --process-delay 0 -p "x"`
- **When:** `clr --retry-on-process 1 --process-delay 0 --max-sessions 0 -p "x"` using fake
- **Then:** Exit 0; stderr contains `[Process]` retry progress line; two invocations
- **Exit:** 0
- **Source:** [046_retry_on_process.md](../../../../docs/cli/param/046_retry_on_process.md)
- **Commands:** run, ask

---

### EC-8: Process retries exhausted → exit 4; [Process] exhaustion in stderr

- **Given:** fake always exits 4; `--retry-on-process 2 --process-delay 0 -p "x"`
- **When:** `clr --retry-on-process 2 --process-delay 0 --max-sessions 0 -p "x"` using fake
- **Then:** Exit 4; stderr contains `[Process]` and "exhausted"; 3 total invocations
- **Exit:** 4
- **Source:** [046_retry_on_process.md](../../../../docs/cli/param/046_retry_on_process.md)
- **Commands:** run, ask

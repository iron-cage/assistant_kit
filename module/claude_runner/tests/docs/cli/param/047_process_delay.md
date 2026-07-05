# Parameter :: `--process-delay`

Edge case coverage for the `--process-delay` parameter (new in retry system redesign).
See [047_process_delay.md](../../../../docs/cli/param/047_process_delay.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--process-delay` | Documentation |
| EC-2 | `--process-delay 0 --dry-run` → exit 0; zero-second delay accepted | Behavioral Divergence |
| EC-3 | `--process-delay 30 --dry-run` → exit 0; non-zero value accepted | Behavioral Divergence |
| EC-4 | `CLR_PROCESS_DELAY=30 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_PROCESS_DELAY=10 --process-delay 30 --dry-run` → CLI 30 wins | CLI-wins |
| EC-6 | `CLR_PROCESS_DELAY=abc --dry-run` → silently ignored | Validation |

## Test Coverage Summary

- Documentation: 1 test (EC-1)
- Behavioral Divergence: 2 tests (EC-2, EC-3)
- Env Var: 1 test (EC-4)
- CLI-wins: 1 test (EC-5)
- Validation: 1 test (EC-6)

**Total:** 6 edge cases

## Architectural Constraint

Delay integration is covered by `046_retry_on_process.md` EC-7 (`--process-delay 0` used there).
These tests verify parse and env-var behavior only via dry-run.

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_process_delay_help_listed` | `retry_process_test.rs` |
| EC-2 | `ec2_process_delay_zero_dry_run` | `retry_process_test.rs` |
| EC-3 | `ec3_process_delay_nonzero_dry_run` | `retry_process_test.rs` |
| EC-4 | `ec4_clr_process_delay_env_var_accepted` | `retry_process_test.rs` |
| EC-5 | `ec5_process_delay_cli_wins_over_env` | `retry_process_test.rs` |
| EC-6 | `ec6_clr_process_delay_invalid_ignored` | `retry_process_test.rs` |

---

### EC-1: --help lists --process-delay

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--process-delay`
- **Exit:** 0
- **Source:** [047_process_delay.md](../../../../docs/cli/param/047_process_delay.md)
- **Commands:** run, ask

---

### EC-2: --process-delay 0 --dry-run → exit 0; zero accepted

- **Given:** `--process-delay 0` and `--dry-run` set
- **When:** `clr --process-delay 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced. **Divergence from EC-3:** 0 = immediate; 30 = 30s sleep
- **Exit:** 0
- **Source:** [047_process_delay.md](../../../../docs/cli/param/047_process_delay.md)
- **Commands:** run, ask

---

### EC-3: --process-delay 30 --dry-run → exit 0; non-zero accepted

- **Given:** `--process-delay 30` and `--dry-run` set
- **When:** `clr --process-delay 30 --dry-run "task"`
- **Then:** Exit 0; flag accepted without error
- **Exit:** 0
- **Source:** [047_process_delay.md](../../../../docs/cli/param/047_process_delay.md)
- **Commands:** run, ask

---

### EC-4: CLR_PROCESS_DELAY=30 env var → applied when CLI flag absent

- **Given:** `CLR_PROCESS_DELAY=30` set; no CLI flag; `--dry-run` set
- **When:** `CLR_PROCESS_DELAY=30 clr --dry-run "task"`
- **Then:** Exit 0; env var accepted
- **Exit:** 0
- **Source:** [047_process_delay.md](../../../../docs/cli/param/047_process_delay.md)
- **Commands:** run, ask

---

### EC-5: --process-delay CLI wins over CLR_PROCESS_DELAY

- **Given:** `CLR_PROCESS_DELAY=10` set; `--process-delay 30` on CLI; `--dry-run` set
- **When:** `CLR_PROCESS_DELAY=10 clr --process-delay 30 --dry-run "task"`
- **Then:** Exit 0; CLI value 30 used
- **Exit:** 0
- **Source:** [047_process_delay.md](../../../../docs/cli/param/047_process_delay.md)
- **Commands:** run, ask

---

### EC-6: CLR_PROCESS_DELAY=invalid → silently ignored

- **Given:** `CLR_PROCESS_DELAY=abc` set; no CLI flag; `--dry-run` set
- **When:** `CLR_PROCESS_DELAY=abc clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored
- **Exit:** 0
- **Source:** [047_process_delay.md](../../../../docs/cli/param/047_process_delay.md)
- **Commands:** run, ask

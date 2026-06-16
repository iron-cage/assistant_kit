# Parameter :: `--runner-delay`

Edge case coverage for the `--runner-delay` parameter (new in retry system redesign).
See [051_runner_delay.md](../../../../docs/cli/param/051_runner_delay.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--runner-delay` | Documentation |
| EC-2 | `--runner-delay 0 --dry-run` → exit 0; zero-second delay accepted | Behavioral Divergence |
| EC-3 | `--runner-delay 30 --dry-run` → exit 0; non-zero value accepted | Behavioral Divergence |
| EC-4 | `CLR_RUNNER_DELAY=30 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_RUNNER_DELAY=10 --runner-delay 30 --dry-run` → CLI 30 wins | CLI-wins |
| EC-6 | `CLR_RUNNER_DELAY=abc --dry-run` → silently ignored | Validation |

## Test Coverage Summary

- Documentation: 1 test (EC-1)
- Behavioral Divergence: 2 tests (EC-2, EC-3)
- Env Var: 1 test (EC-4)
- CLI-wins: 1 test (EC-5)
- Validation: 1 test (EC-6)

**Total:** 6 edge cases

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_runner_delay_help_listed` | `retry_runner_test.rs` |
| EC-2 | `ec2_runner_delay_zero_dry_run` | `retry_runner_test.rs` |
| EC-3 | `ec3_runner_delay_nonzero_dry_run` | `retry_runner_test.rs` |
| EC-4 | `ec4_clr_runner_delay_env_var_accepted` | `retry_runner_test.rs` |
| EC-5 | `ec5_runner_delay_cli_wins_over_env` | `retry_runner_test.rs` |
| EC-6 | `ec6_clr_runner_delay_invalid_ignored` | `retry_runner_test.rs` |

---

### EC-1: --help lists --runner-delay

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--runner-delay`
- **Exit:** 0
- **Source:** [051_runner_delay.md](../../../../docs/cli/param/051_runner_delay.md)
- **Commands:** run, ask

---

### EC-2: --runner-delay 0 --dry-run → exit 0; zero accepted

- **Given:** `--runner-delay 0` and `--dry-run` set
- **When:** `clr --runner-delay 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced. **Divergence from EC-3:** 0 = immediate; 30 = 30s sleep
- **Exit:** 0
- **Source:** [051_runner_delay.md](../../../../docs/cli/param/051_runner_delay.md)
- **Commands:** run, ask

---

### EC-3: --runner-delay 30 --dry-run → exit 0; non-zero accepted

- **Given:** `--runner-delay 30` and `--dry-run` set
- **When:** `clr --runner-delay 30 --dry-run "task"`
- **Then:** Exit 0; flag accepted without error
- **Exit:** 0
- **Source:** [051_runner_delay.md](../../../../docs/cli/param/051_runner_delay.md)
- **Commands:** run, ask

---

### EC-4: CLR_RUNNER_DELAY=30 env var → applied when CLI flag absent

- **Given:** `CLR_RUNNER_DELAY=30` set; no CLI flag; `--dry-run` set
- **When:** `CLR_RUNNER_DELAY=30 clr --dry-run "task"`
- **Then:** Exit 0; env var accepted
- **Exit:** 0
- **Source:** [051_runner_delay.md](../../../../docs/cli/param/051_runner_delay.md)
- **Commands:** run, ask

---

### EC-5: --runner-delay CLI wins over CLR_RUNNER_DELAY

- **Given:** `CLR_RUNNER_DELAY=10` set; `--runner-delay 30` on CLI; `--dry-run` set
- **When:** `CLR_RUNNER_DELAY=10 clr --runner-delay 30 --dry-run "task"`
- **Then:** Exit 0; CLI value 30 used
- **Exit:** 0
- **Source:** [051_runner_delay.md](../../../../docs/cli/param/051_runner_delay.md)
- **Commands:** run, ask

---

### EC-6: CLR_RUNNER_DELAY=invalid → silently ignored

- **Given:** `CLR_RUNNER_DELAY=abc` set; no CLI flag; `--dry-run` set
- **When:** `CLR_RUNNER_DELAY=abc clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored
- **Exit:** 0
- **Source:** [051_runner_delay.md](../../../../docs/cli/param/051_runner_delay.md)
- **Commands:** run, ask

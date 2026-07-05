# Parameter :: `--validation-delay`

Edge case coverage for the `--validation-delay` parameter (new in retry system redesign).
See [049_validation_delay.md](../../../../docs/cli/param/049_validation_delay.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--validation-delay` | Documentation |
| EC-2 | `--validation-delay 0 --dry-run` → exit 0; zero-second delay accepted | Behavioral Divergence |
| EC-3 | `--validation-delay 30 --dry-run` → exit 0; non-zero value accepted | Behavioral Divergence |
| EC-4 | `CLR_VALIDATION_DELAY=30 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_VALIDATION_DELAY=10 --validation-delay 30 --dry-run` → CLI 30 wins | CLI-wins |
| EC-6 | `CLR_VALIDATION_DELAY=abc --dry-run` → silently ignored | Validation |

## Test Coverage Summary

- Documentation: 1 test (EC-1)
- Behavioral Divergence: 2 tests (EC-2, EC-3)
- Env Var: 1 test (EC-4)
- CLI-wins: 1 test (EC-5)
- Validation: 1 test (EC-6)

**Total:** 6 edge cases

## Architectural Constraint

Delay integration is covered by `048_retry_on_validation.md` EC-7 (`--validation-delay 0` used there).
These tests verify parse and env-var behavior only via dry-run. No predecessor flag existed
for validation delay — this param is entirely new.

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_validation_delay_help_listed` | `retry_validation_test.rs` |
| EC-2 | `ec2_validation_delay_zero_dry_run` | `retry_validation_test.rs` |
| EC-3 | `ec3_validation_delay_nonzero_dry_run` | `retry_validation_test.rs` |
| EC-4 | `ec4_clr_validation_delay_env_var_accepted` | `retry_validation_test.rs` |
| EC-5 | `ec5_validation_delay_cli_wins_over_env` | `retry_validation_test.rs` |
| EC-6 | `ec6_clr_validation_delay_invalid_ignored` | `retry_validation_test.rs` |

---

### EC-1: --help lists --validation-delay

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--validation-delay`
- **Exit:** 0
- **Source:** [049_validation_delay.md](../../../../docs/cli/param/049_validation_delay.md)
- **Commands:** run, ask

---

### EC-2: --validation-delay 0 --dry-run → exit 0; zero accepted

- **Given:** `--validation-delay 0` and `--dry-run` set
- **When:** `clr --validation-delay 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced. **Divergence from EC-3:** 0 = immediate; 30 = 30s sleep
- **Exit:** 0
- **Source:** [049_validation_delay.md](../../../../docs/cli/param/049_validation_delay.md)
- **Commands:** run, ask

---

### EC-3: --validation-delay 30 --dry-run → exit 0; non-zero accepted

- **Given:** `--validation-delay 30` and `--dry-run` set
- **When:** `clr --validation-delay 30 --dry-run "task"`
- **Then:** Exit 0; flag accepted without error
- **Exit:** 0
- **Source:** [049_validation_delay.md](../../../../docs/cli/param/049_validation_delay.md)
- **Commands:** run, ask

---

### EC-4: CLR_VALIDATION_DELAY=30 env var → applied when CLI flag absent

- **Given:** `CLR_VALIDATION_DELAY=30` set; no CLI flag; `--dry-run` set
- **When:** `CLR_VALIDATION_DELAY=30 clr --dry-run "task"`
- **Then:** Exit 0; env var accepted
- **Exit:** 0
- **Source:** [049_validation_delay.md](../../../../docs/cli/param/049_validation_delay.md)
- **Commands:** run, ask

---

### EC-5: --validation-delay CLI wins over CLR_VALIDATION_DELAY

- **Given:** `CLR_VALIDATION_DELAY=10` set; `--validation-delay 30` on CLI; `--dry-run` set
- **When:** `CLR_VALIDATION_DELAY=10 clr --validation-delay 30 --dry-run "task"`
- **Then:** Exit 0; CLI value 30 used
- **Exit:** 0
- **Source:** [049_validation_delay.md](../../../../docs/cli/param/049_validation_delay.md)
- **Commands:** run, ask

---

### EC-6: CLR_VALIDATION_DELAY=invalid → silently ignored

- **Given:** `CLR_VALIDATION_DELAY=abc` set; no CLI flag; `--dry-run` set
- **When:** `CLR_VALIDATION_DELAY=abc clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored
- **Exit:** 0
- **Source:** [049_validation_delay.md](../../../../docs/cli/param/049_validation_delay.md)
- **Commands:** run, ask

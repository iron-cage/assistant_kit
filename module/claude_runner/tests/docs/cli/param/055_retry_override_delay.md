# Parameter :: `--retry-override-delay`

Edge case coverage for the `--retry-override-delay` parameter (Tier 1 delay: overrides all class-specific delays).
See [055_retry_override_delay.md](../../../../docs/cli/param/055_retry_override_delay.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--retry-override-delay` | Documentation |
| EC-2 | `--retry-override-delay 0 --dry-run` → exit 0; zero-second delay accepted | Behavioral Divergence |
| EC-3 | `--retry-override-delay 30 --dry-run` → exit 0; non-zero value accepted | Behavioral Divergence |
| EC-4 | `CLR_RETRY_OVERRIDE_DELAY=30 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_RETRY_OVERRIDE_DELAY=10 --retry-override-delay 30 --dry-run` → CLI 30 wins | CLI-wins |
| EC-6 | `CLR_RETRY_OVERRIDE_DELAY=abc --dry-run` → silently ignored | Validation |

## Test Coverage Summary

- Documentation: 1 test (EC-1)
- Behavioral Divergence: 2 tests (EC-2, EC-3)
- Env Var: 1 test (EC-4)
- CLI-wins: 1 test (EC-5)
- Validation: 1 test (EC-6)

**Total:** 6 edge cases

## Architectural Constraint

`--retry-override-delay` is Tier 1 in the delay resolution chain: when set, it overrides the
delay for ALL error classes. Delay integration is demonstrated by `54_retry_override.md`
EC-8/EC-9 (using `--transient-delay 0` / `--service-delay 0` because override-delay=0 also works).
These tests verify parse and env-var behavior only via dry-run.

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_retry_override_delay_help_listed` | `retry_override_test.rs` |
| EC-2 | `ec2_retry_override_delay_zero_dry_run` | `retry_override_test.rs` |
| EC-3 | `ec3_retry_override_delay_nonzero_dry_run` | `retry_override_test.rs` |
| EC-4 | `ec4_clr_retry_override_delay_env_var_accepted` | `retry_override_test.rs` |
| EC-5 | `ec5_retry_override_delay_cli_wins_over_env` | `retry_override_test.rs` |
| EC-6 | `ec6_clr_retry_override_delay_invalid_ignored` | `retry_override_test.rs` |

---

### EC-1: --help lists --retry-override-delay

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--retry-override-delay`
- **Exit:** 0
- **Source:** [055_retry_override_delay.md](../../../../docs/cli/param/055_retry_override_delay.md)
- **Commands:** run, ask

---

### EC-2: --retry-override-delay 0 --dry-run → exit 0; zero accepted

- **Given:** `--retry-override-delay 0` and `--dry-run` set
- **When:** `clr --retry-override-delay 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced. **Divergence from EC-3:** 0 = immediate; 30 = 30s sleep before each retry
- **Exit:** 0
- **Source:** [055_retry_override_delay.md](../../../../docs/cli/param/055_retry_override_delay.md)
- **Commands:** run, ask

---

### EC-3: --retry-override-delay 30 --dry-run → exit 0; non-zero accepted

- **Given:** `--retry-override-delay 30` and `--dry-run` set
- **When:** `clr --retry-override-delay 30 --dry-run "task"`
- **Then:** Exit 0; flag accepted without error
- **Exit:** 0
- **Source:** [055_retry_override_delay.md](../../../../docs/cli/param/055_retry_override_delay.md)
- **Commands:** run, ask

---

### EC-4: CLR_RETRY_OVERRIDE_DELAY=30 env var → applied when CLI flag absent

- **Given:** `CLR_RETRY_OVERRIDE_DELAY=30` set; no CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_OVERRIDE_DELAY=30 clr --dry-run "task"`
- **Then:** Exit 0; env var accepted
- **Exit:** 0
- **Source:** [055_retry_override_delay.md](../../../../docs/cli/param/055_retry_override_delay.md)
- **Commands:** run, ask

---

### EC-5: --retry-override-delay CLI wins over CLR_RETRY_OVERRIDE_DELAY

- **Given:** `CLR_RETRY_OVERRIDE_DELAY=10` set; `--retry-override-delay 30` on CLI; `--dry-run` set
- **When:** `CLR_RETRY_OVERRIDE_DELAY=10 clr --retry-override-delay 30 --dry-run "task"`
- **Then:** Exit 0; CLI value 30 used
- **Exit:** 0
- **Source:** [055_retry_override_delay.md](../../../../docs/cli/param/055_retry_override_delay.md)
- **Commands:** run, ask

---

### EC-6: CLR_RETRY_OVERRIDE_DELAY=invalid → silently ignored

- **Given:** `CLR_RETRY_OVERRIDE_DELAY=abc` set; no CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_OVERRIDE_DELAY=abc clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored
- **Exit:** 0
- **Source:** [055_retry_override_delay.md](../../../../docs/cli/param/055_retry_override_delay.md)
- **Commands:** run, ask

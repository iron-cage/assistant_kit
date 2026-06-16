# Parameter :: `--retry-on-runner`
<!-- BUG-299: Architectural Constraint (see below) is a known incomplete implementation — filed as bug -->

Edge case coverage for the `--retry-on-runner` parameter (new in retry system redesign).
See [050_retry_on_runner.md](../../../../docs/cli/param/050_retry_on_runner.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--retry-on-runner` | Documentation |
| EC-2 | `--retry-on-runner 0 --dry-run` → exit 0; explicit zero accepted | Behavioral Divergence |
| EC-3 | `--retry-on-runner 2 --dry-run` → exit 0; nonzero accepted | Behavioral Divergence |
| EC-4 | `CLR_RETRY_ON_RUNNER=2 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_RETRY_ON_RUNNER=1 --retry-on-runner 3 --dry-run` → CLI 3 wins | CLI-wins |
| EC-6 | `CLR_RETRY_ON_RUNNER=notanumber --dry-run` → silently ignored | Validation |

## Test Coverage Summary

- Documentation: 1 test (EC-1)
- Behavioral Divergence: 2 tests (EC-2, EC-3)
- Env Var: 1 test (EC-4)
- CLI-wins: 1 test (EC-5)
- Validation: 1 test (EC-6)

**Total:** 6 edge cases

## Architectural Constraint

Runner class errors (binary not found, spawn failed, gate timeout) cause the runner to exit
before the retry loop is entered. `--retry-on-runner` is accepted at parse time and stored,
but has NO runtime effect in the current implementation. Integration tests demonstrating
retry behavior for this class cannot be constructed. These tests verify parse and env-var
behavior only.

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_retry_on_runner_help_listed` | `retry_runner_test.rs` |
| EC-2 | `ec2_retry_on_runner_zero_dry_run` | `retry_runner_test.rs` |
| EC-3 | `ec3_retry_on_runner_nonzero_dry_run` | `retry_runner_test.rs` |
| EC-4 | `ec4_clr_retry_on_runner_env_var_accepted` | `retry_runner_test.rs` |
| EC-5 | `ec5_retry_on_runner_cli_wins_over_env` | `retry_runner_test.rs` |
| EC-6 | `ec6_clr_retry_on_runner_invalid_ignored` | `retry_runner_test.rs` |

---

### EC-1: --help lists --retry-on-runner

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--retry-on-runner`
- **Exit:** 0
- **Source:** [050_retry_on_runner.md](../../../../docs/cli/param/050_retry_on_runner.md)
- **Commands:** run, ask

---

### EC-2: --retry-on-runner 0 --dry-run → exit 0; explicit zero accepted

- **Given:** `--retry-on-runner 0` and `--dry-run` set
- **When:** `clr --retry-on-runner 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced. **Divergence from EC-3:** 0 is stored as zero; 2 is stored as two; both accepted without error
- **Exit:** 0
- **Source:** [050_retry_on_runner.md](../../../../docs/cli/param/050_retry_on_runner.md)
- **Commands:** run, ask

---

### EC-3: --retry-on-runner 2 --dry-run → exit 0; nonzero accepted

- **Given:** `--retry-on-runner 2` and `--dry-run` set
- **When:** `clr --retry-on-runner 2 --dry-run "task"`
- **Then:** Exit 0; flag accepted without error
- **Exit:** 0
- **Source:** [050_retry_on_runner.md](../../../../docs/cli/param/050_retry_on_runner.md)
- **Commands:** run, ask

---

### EC-4: CLR_RETRY_ON_RUNNER=2 env var → applied when CLI flag absent

- **Given:** `CLR_RETRY_ON_RUNNER=2` set; no CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_ON_RUNNER=2 clr --dry-run "task"`
- **Then:** Exit 0; env var accepted
- **Exit:** 0
- **Source:** [050_retry_on_runner.md](../../../../docs/cli/param/050_retry_on_runner.md)
- **Commands:** run, ask

---

### EC-5: CLI wins over CLR_RETRY_ON_RUNNER

- **Given:** `CLR_RETRY_ON_RUNNER=1` set; `--retry-on-runner 3` on CLI; `--dry-run` set
- **When:** `CLR_RETRY_ON_RUNNER=1 clr --retry-on-runner 3 --dry-run "task"`
- **Then:** Exit 0; CLI value 3 used
- **Exit:** 0
- **Source:** [050_retry_on_runner.md](../../../../docs/cli/param/050_retry_on_runner.md)
- **Commands:** run, ask

---

### EC-6: CLR_RETRY_ON_RUNNER=invalid → silently ignored

- **Given:** `CLR_RETRY_ON_RUNNER=notanumber` set; no CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_ON_RUNNER=notanumber clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored
- **Exit:** 0
- **Source:** [050_retry_on_runner.md](../../../../docs/cli/param/050_retry_on_runner.md)
- **Commands:** run, ask

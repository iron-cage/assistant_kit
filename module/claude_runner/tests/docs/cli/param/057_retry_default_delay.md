# Parameter :: `--retry-default-delay`

Edge case coverage for the `--retry-default-delay` parameter (Tier 3 delay: fallback when no override or class-specific delay is set).
See [057_retry_default_delay.md](../../../../docs/cli/param/057_retry_default_delay.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--retry-default-delay` | Documentation |
| EC-2 | `--retry-default-delay 0 --dry-run` → exit 0; zero-second delay accepted | Behavioral Divergence |
| EC-3 | `--retry-default-delay 30 --dry-run` → exit 0; non-zero value accepted | Behavioral Divergence |
| EC-4 | `CLR_RETRY_DEFAULT_DELAY=30 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_RETRY_DEFAULT_DELAY=10 --retry-default-delay 30 --dry-run` → CLI 30 wins | CLI-wins |
| EC-6 | `CLR_RETRY_DEFAULT_DELAY=abc --dry-run` → silently ignored | Validation |

## Test Coverage Summary

- Documentation: 1 test (EC-1)
- Behavioral Divergence: 2 tests (EC-2, EC-3)
- Env Var: 1 test (EC-4)
- CLI-wins: 1 test (EC-5)
- Validation: 1 test (EC-6)

**Total:** 6 edge cases

## Architectural Constraint

`--retry-default-delay` is Tier 3 in the delay resolution chain. Delay integration is
demonstrated across the per-class EC-10 tests (e.g., `34_retry_on_transient.md` EC-10,
`44_retry_on_service.md` EC-10) which use `--retry-default-delay 0` to avoid sleep in
fallback-default integration tests. The built-in default is 30s (applies when
`--retry-default-delay` is also absent). These tests verify parse and env-var behavior
only via dry-run.

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_retry_default_delay_help_listed` | `retry_default_test.rs` |
| EC-2 | `ec2_retry_default_delay_zero_dry_run` | `retry_default_test.rs` |
| EC-3 | `ec3_retry_default_delay_nonzero_dry_run` | `retry_default_test.rs` |
| EC-4 | `ec4_clr_retry_default_delay_env_var_accepted` | `retry_default_test.rs` |
| EC-5 | `ec5_retry_default_delay_cli_wins_over_env` | `retry_default_test.rs` |
| EC-6 | `ec6_clr_retry_default_delay_invalid_ignored` | `retry_default_test.rs` |

---

### EC-1: --help lists --retry-default-delay

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--retry-default-delay`
- **Exit:** 0
- **Source:** [057_retry_default_delay.md](../../../../docs/cli/param/057_retry_default_delay.md)
- **Commands:** run, ask

---

### EC-2: --retry-default-delay 0 --dry-run → exit 0; zero accepted

- **Given:** `--retry-default-delay 0` and `--dry-run` set
- **When:** `clr --retry-default-delay 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced. **Divergence from EC-3:** 0 = immediate retry; 30 = 30s sleep before each fallback retry
- **Exit:** 0
- **Source:** [057_retry_default_delay.md](../../../../docs/cli/param/057_retry_default_delay.md)
- **Commands:** run, ask

---

### EC-3: --retry-default-delay 30 --dry-run → exit 0; non-zero accepted

- **Given:** `--retry-default-delay 30` and `--dry-run` set
- **When:** `clr --retry-default-delay 30 --dry-run "task"`
- **Then:** Exit 0; flag accepted without error
- **Exit:** 0
- **Source:** [057_retry_default_delay.md](../../../../docs/cli/param/057_retry_default_delay.md)
- **Commands:** run, ask

---

### EC-4: CLR_RETRY_DEFAULT_DELAY=30 env var → applied when CLI flag absent

- **Given:** `CLR_RETRY_DEFAULT_DELAY=30` set; no CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_DEFAULT_DELAY=30 clr --dry-run "task"`
- **Then:** Exit 0; env var accepted
- **Exit:** 0
- **Source:** [057_retry_default_delay.md](../../../../docs/cli/param/057_retry_default_delay.md)
- **Commands:** run, ask

---

### EC-5: --retry-default-delay CLI wins over CLR_RETRY_DEFAULT_DELAY

- **Given:** `CLR_RETRY_DEFAULT_DELAY=10` set; `--retry-default-delay 30` on CLI; `--dry-run` set
- **When:** `CLR_RETRY_DEFAULT_DELAY=10 clr --retry-default-delay 30 --dry-run "task"`
- **Then:** Exit 0; CLI value 30 used
- **Exit:** 0
- **Source:** [057_retry_default_delay.md](../../../../docs/cli/param/057_retry_default_delay.md)
- **Commands:** run, ask

---

### EC-6: CLR_RETRY_DEFAULT_DELAY=invalid → silently ignored

- **Given:** `CLR_RETRY_DEFAULT_DELAY=abc` set; no CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_DEFAULT_DELAY=abc clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored
- **Exit:** 0
- **Source:** [057_retry_default_delay.md](../../../../docs/cli/param/057_retry_default_delay.md)
- **Commands:** run, ask

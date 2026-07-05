# Parameter :: `--unknown-delay`

Edge case coverage for the `--unknown-delay` parameter (new in retry system redesign).
See [053_unknown_delay.md](../../../../docs/cli/param/053_unknown_delay.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--unknown-delay` | Documentation |
| EC-2 | `--unknown-delay 0 --dry-run` → exit 0; zero-second delay accepted | Behavioral Divergence |
| EC-3 | `--unknown-delay 30 --dry-run` → exit 0; non-zero value accepted | Behavioral Divergence |
| EC-4 | `CLR_UNKNOWN_DELAY=30 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_UNKNOWN_DELAY=10 --unknown-delay 30 --dry-run` → CLI 30 wins | CLI-wins |
| EC-6 | `CLR_UNKNOWN_DELAY=abc --dry-run` → silently ignored | Validation |

## Test Coverage Summary

- Documentation: 1 test (EC-1)
- Behavioral Divergence: 2 tests (EC-2, EC-3)
- Env Var: 1 test (EC-4)
- CLI-wins: 1 test (EC-5)
- Validation: 1 test (EC-6)

**Total:** 6 edge cases

## Architectural Constraint

Delay integration is covered by `052_retry_on_unknown.md` EC-7 (`--unknown-delay 0` used there).
These tests verify parse and env-var behavior only via dry-run. No predecessor delay flag
existed for unknown errors — this param is entirely new.

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_unknown_delay_help_listed` | `retry_unknown_test.rs` |
| EC-2 | `ec2_unknown_delay_zero_dry_run` | `retry_unknown_test.rs` |
| EC-3 | `ec3_unknown_delay_nonzero_dry_run` | `retry_unknown_test.rs` |
| EC-4 | `ec4_clr_unknown_delay_env_var_accepted` | `retry_unknown_test.rs` |
| EC-5 | `ec5_unknown_delay_cli_wins_over_env` | `retry_unknown_test.rs` |
| EC-6 | `ec6_clr_unknown_delay_invalid_ignored` | `retry_unknown_test.rs` |

---

### EC-1: --help lists --unknown-delay

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--unknown-delay`
- **Exit:** 0
- **Source:** [053_unknown_delay.md](../../../../docs/cli/param/053_unknown_delay.md)
- **Commands:** run, ask

---

### EC-2: --unknown-delay 0 --dry-run → exit 0; zero accepted

- **Given:** `--unknown-delay 0` and `--dry-run` set
- **When:** `clr --unknown-delay 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced. **Divergence from EC-3:** 0 = immediate; 30 = 30s sleep
- **Exit:** 0
- **Source:** [053_unknown_delay.md](../../../../docs/cli/param/053_unknown_delay.md)
- **Commands:** run, ask

---

### EC-3: --unknown-delay 30 --dry-run → exit 0; non-zero accepted

- **Given:** `--unknown-delay 30` and `--dry-run` set
- **When:** `clr --unknown-delay 30 --dry-run "task"`
- **Then:** Exit 0; flag accepted without error
- **Exit:** 0
- **Source:** [053_unknown_delay.md](../../../../docs/cli/param/053_unknown_delay.md)
- **Commands:** run, ask

---

### EC-4: CLR_UNKNOWN_DELAY=30 env var → applied when CLI flag absent

- **Given:** `CLR_UNKNOWN_DELAY=30` set; no CLI flag; `--dry-run` set
- **When:** `CLR_UNKNOWN_DELAY=30 clr --dry-run "task"`
- **Then:** Exit 0; env var accepted
- **Exit:** 0
- **Source:** [053_unknown_delay.md](../../../../docs/cli/param/053_unknown_delay.md)
- **Commands:** run, ask

---

### EC-5: --unknown-delay CLI wins over CLR_UNKNOWN_DELAY

- **Given:** `CLR_UNKNOWN_DELAY=10` set; `--unknown-delay 30` on CLI; `--dry-run` set
- **When:** `CLR_UNKNOWN_DELAY=10 clr --unknown-delay 30 --dry-run "task"`
- **Then:** Exit 0; CLI value 30 used
- **Exit:** 0
- **Source:** [053_unknown_delay.md](../../../../docs/cli/param/053_unknown_delay.md)
- **Commands:** run, ask

---

### EC-6: CLR_UNKNOWN_DELAY=invalid → silently ignored

- **Given:** `CLR_UNKNOWN_DELAY=abc` set; no CLI flag; `--dry-run` set
- **When:** `CLR_UNKNOWN_DELAY=abc clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored
- **Exit:** 0
- **Source:** [053_unknown_delay.md](../../../../docs/cli/param/053_unknown_delay.md)
- **Commands:** run, ask

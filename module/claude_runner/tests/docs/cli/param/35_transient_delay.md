# Parameter :: `--transient-delay`

Edge case coverage for the `--transient-delay` parameter (renamed from `--retry-delay`).
See [035_transient_delay.md](../../../../docs/cli/param/035_transient_delay.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--transient-delay` | Documentation |
| EC-2 | `--transient-delay 0 --dry-run` → exit 0; zero-second delay accepted | Behavioral Divergence |
| EC-3 | `--transient-delay 30 --dry-run` → exit 0; non-zero value accepted | Behavioral Divergence |
| EC-4 | `CLR_TRANSIENT_DELAY=30 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_TRANSIENT_DELAY=10 --transient-delay 30 --dry-run` → CLI 30 wins | CLI-wins |
| EC-6 | `CLR_TRANSIENT_DELAY=abc --dry-run` → silently ignored | Validation |
| EC-7 | Old flag `--retry-delay` rejected → exit 1; "unknown option" | Behavioral Divergence |

## Test Coverage Summary

- Documentation: 1 test (EC-1)
- Behavioral Divergence: 3 tests (EC-2, EC-3, EC-7)
- Env Var: 1 test (EC-4)
- CLI-wins: 1 test (EC-5)
- Validation: 1 test (EC-6)

**Total:** 7 edge cases

## Architectural Constraint

Delay behavior (sleeping N seconds between retries) cannot be exercised at realistic durations.
Integration tests use `--transient-delay 0` to confirm immediate retry. The old flag name
`--retry-delay` is now an unknown option and must be confirmed rejected (EC-7).

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_transient_delay_help_listed` | `retry_transient_test.rs` |
| EC-2 | `ec2_transient_delay_zero_dry_run` | `retry_transient_test.rs` |
| EC-3 | `ec3_transient_delay_nonzero_dry_run` | `retry_transient_test.rs` |
| EC-4 | `ec4_clr_transient_delay_env_var_accepted` | `retry_transient_test.rs` |
| EC-5 | `ec5_transient_delay_cli_wins_over_env` | `retry_transient_test.rs` |
| EC-6 | `ec6_clr_transient_delay_invalid_ignored` | `retry_transient_test.rs` |
| EC-7 | `ec7_old_flag_retry_delay_rejected` | `retry_transient_test.rs` |

---

### EC-1: --help lists --transient-delay

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--transient-delay`; does NOT contain `--retry-delay`
- **Exit:** 0
- **Source:** [035_transient_delay.md](../../../../docs/cli/param/035_transient_delay.md)
- **Commands:** run, ask

---

### EC-2: --transient-delay 0 --dry-run → exit 0; zero-second delay accepted

- **Given:** `--transient-delay 0` and `--dry-run` set
- **When:** `clr --transient-delay 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced; flag accepted without error. **Divergence from EC-3:** 0 means immediate retry; 30 (EC-3) introduces 30s sleep per retry
- **Exit:** 0
- **Source:** [035_transient_delay.md](../../../../docs/cli/param/035_transient_delay.md)
- **Commands:** run, ask

---

### EC-3: --transient-delay 30 --dry-run → exit 0; non-zero value accepted

- **Given:** `--transient-delay 30` and `--dry-run` set
- **When:** `clr --transient-delay 30 --dry-run "task"`
- **Then:** Exit 0; flag accepted without error
- **Exit:** 0
- **Source:** [035_transient_delay.md](../../../../docs/cli/param/035_transient_delay.md)
- **Commands:** run, ask

---

### EC-4: CLR_TRANSIENT_DELAY=30 env var → applied when CLI flag absent

- **Given:** `CLR_TRANSIENT_DELAY=30` set; no `--transient-delay` CLI flag; `--dry-run` set
- **When:** `CLR_TRANSIENT_DELAY=30 clr --dry-run "task"`
- **Then:** Exit 0; env var accepted
- **Exit:** 0
- **Source:** [035_transient_delay.md](../../../../docs/cli/param/035_transient_delay.md)
- **Commands:** run, ask

---

### EC-5: --transient-delay CLI wins over CLR_TRANSIENT_DELAY

- **Given:** `CLR_TRANSIENT_DELAY=10` set; `--transient-delay 30` on CLI; `--dry-run` set
- **When:** `CLR_TRANSIENT_DELAY=10 clr --transient-delay 30 --dry-run "task"`
- **Then:** Exit 0; CLI value 30 used; env var 10 ignored
- **Exit:** 0
- **Source:** [035_transient_delay.md](../../../../docs/cli/param/035_transient_delay.md)
- **Commands:** run, ask

---

### EC-6: CLR_TRANSIENT_DELAY=invalid → silently ignored

- **Given:** `CLR_TRANSIENT_DELAY=abc` set; no CLI flag; `--dry-run` set
- **When:** `CLR_TRANSIENT_DELAY=abc clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored; fallback default delay applies
- **Exit:** 0
- **Source:** [035_transient_delay.md](../../../../docs/cli/param/035_transient_delay.md)
- **Commands:** run, ask

---

### EC-7: Old flag name --retry-delay rejected at parse time

- **Given:** `--retry-delay 0` (old flag name) passed
- **When:** `clr --retry-delay 0 --dry-run "task"`
- **Then:** Exit 1; stderr contains "unknown option"; parse aborted
- **Exit:** 1
- **Source:** [035_transient_delay.md](../../../../docs/cli/param/035_transient_delay.md)
- **Commands:** run, ask

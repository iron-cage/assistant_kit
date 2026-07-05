# Parameter :: `--service-delay`

Edge case coverage for the `--service-delay` parameter (renamed from `--api-error-delay`).
See [045_service_delay.md](../../../../docs/cli/param/045_service_delay.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--service-delay` | Documentation |
| EC-2 | `--service-delay 0 --dry-run` → exit 0; zero-second delay accepted | Behavioral Divergence |
| EC-3 | `--service-delay 30 --dry-run` → exit 0; non-zero value accepted | Behavioral Divergence |
| EC-4 | `CLR_SERVICE_DELAY=30 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_SERVICE_DELAY=10 --service-delay 30 --dry-run` → CLI 30 wins | CLI-wins |
| EC-6 | `CLR_SERVICE_DELAY=abc --dry-run` → silently ignored | Validation |
| EC-7 | Old flag `--api-error-delay` rejected → exit 1; "unknown option" | Behavioral Divergence |

## Test Coverage Summary

- Documentation: 1 test (EC-1)
- Behavioral Divergence: 3 tests (EC-2, EC-3, EC-7)
- Env Var: 1 test (EC-4)
- CLI-wins: 1 test (EC-5)
- Validation: 1 test (EC-6)

**Total:** 7 edge cases

## Architectural Constraint

Delay integration is covered by `044_retry_on_service.md` EC-7 (`--service-delay 0` used there).
These tests verify parse and env-var behavior only via dry-run. Old flag `--api-error-delay`
must be confirmed rejected (EC-7).

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_service_delay_help_listed` | `retry_service_test.rs` |
| EC-2 | `ec2_service_delay_zero_dry_run` | `retry_service_test.rs` |
| EC-3 | `ec3_service_delay_nonzero_dry_run` | `retry_service_test.rs` |
| EC-4 | `ec4_clr_service_delay_env_var_accepted` | `retry_service_test.rs` |
| EC-5 | `ec5_service_delay_cli_wins_over_env` | `retry_service_test.rs` |
| EC-6 | `ec6_clr_service_delay_invalid_ignored` | `retry_service_test.rs` |
| EC-7 | `ec7_old_flag_api_error_delay_rejected` | `retry_service_test.rs` |

---

### EC-1: --help lists --service-delay

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--service-delay`; does NOT contain `--api-error-delay`
- **Exit:** 0
- **Source:** [045_service_delay.md](../../../../docs/cli/param/045_service_delay.md)
- **Commands:** run, ask

---

### EC-2: --service-delay 0 --dry-run → exit 0; zero accepted

- **Given:** `--service-delay 0` and `--dry-run` set
- **When:** `clr --service-delay 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced. **Divergence from EC-3:** 0 = immediate; 30 = 30s sleep
- **Exit:** 0
- **Source:** [045_service_delay.md](../../../../docs/cli/param/045_service_delay.md)
- **Commands:** run, ask

---

### EC-3: --service-delay 30 --dry-run → exit 0; non-zero accepted

- **Given:** `--service-delay 30` and `--dry-run` set
- **When:** `clr --service-delay 30 --dry-run "task"`
- **Then:** Exit 0; flag accepted without error
- **Exit:** 0
- **Source:** [045_service_delay.md](../../../../docs/cli/param/045_service_delay.md)
- **Commands:** run, ask

---

### EC-4: CLR_SERVICE_DELAY=30 env var → applied when CLI flag absent

- **Given:** `CLR_SERVICE_DELAY=30` set; no CLI flag; `--dry-run` set
- **When:** `CLR_SERVICE_DELAY=30 clr --dry-run "task"`
- **Then:** Exit 0; env var accepted
- **Exit:** 0
- **Source:** [045_service_delay.md](../../../../docs/cli/param/045_service_delay.md)
- **Commands:** run, ask

---

### EC-5: --service-delay CLI wins over CLR_SERVICE_DELAY

- **Given:** `CLR_SERVICE_DELAY=10` set; `--service-delay 30` on CLI; `--dry-run` set
- **When:** `CLR_SERVICE_DELAY=10 clr --service-delay 30 --dry-run "task"`
- **Then:** Exit 0; CLI value 30 used
- **Exit:** 0
- **Source:** [045_service_delay.md](../../../../docs/cli/param/045_service_delay.md)
- **Commands:** run, ask

---

### EC-6: CLR_SERVICE_DELAY=invalid → silently ignored

- **Given:** `CLR_SERVICE_DELAY=abc` set; no CLI flag; `--dry-run` set
- **When:** `CLR_SERVICE_DELAY=abc clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored
- **Exit:** 0
- **Source:** [045_service_delay.md](../../../../docs/cli/param/045_service_delay.md)
- **Commands:** run, ask

---

### EC-7: Old flag name --api-error-delay rejected at parse time

- **Given:** `--api-error-delay 30` (old flag name) passed
- **When:** `clr --api-error-delay 30 --dry-run "task"`
- **Then:** Exit 1; stderr contains "unknown option"; parse aborted
- **Exit:** 1
- **Source:** [045_service_delay.md](../../../../docs/cli/param/045_service_delay.md)
- **Commands:** run, ask

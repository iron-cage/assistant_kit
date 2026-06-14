# Parameter :: `--api-error-delay`

Edge case coverage for the `--api-error-delay` parameter. See [038_api_error_delay.md](../../../../docs/cli/param/038_api_error_delay.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--api-error-delay` | Documentation |
| EC-2 | `--api-error-delay 0 --dry-run` → exit 0; zero-second delay accepted | Behavioral Divergence |
| EC-3 | `--api-error-delay 30 --dry-run` → exit 0; default 30s accepted | Behavioral Divergence |
| EC-4 | `CLR_API_ERROR_DELAY=5 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_API_ERROR_DELAY=10 --api-error-delay 30 --dry-run` → CLI 30 wins | CLI-wins |
| EC-6 | `CLR_API_ERROR_DELAY=abc --dry-run` → silently ignored; default 30 used | Validation |
| EC-7 | Fake exits API error once then 0; `--retry-on-api-error 1 --api-error-delay 0` → retry fires immediately; exit 0 | Integration |

## Test Coverage Summary

- Documentation: 1 test (EC-1)
- Behavioral Divergence: 2 tests (EC-2, EC-3)
- Env Var: 1 test (EC-4)
- CLI-wins: 1 test (EC-5)
- Validation: 1 test (EC-6)
- Integration: 1 test (EC-7)

**Total:** 7 edge cases

## Architectural Constraint

The delay behavior (sleeping N seconds between ApiError retry attempts) cannot be exercised in
automated tests at realistic durations — tests use `--api-error-delay 0` to make the retry fire
immediately. EC-7 verifies that delay=0 produces a correct immediate retry rather than testing the
sleep duration itself. The default 30s delay is verified at the parse level only (EC-3) since
exercising a 30-second sleep in a test would be impractical. The `--api-error-delay` parameter is
only meaningful when `--retry-on-api-error` is non-zero; without it, the delay value is unused.

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_api_error_delay_help_listed` | `retry_api_error_test.rs` |
| EC-2 | `ec2_api_error_delay_zero_dry_run` | `retry_api_error_test.rs` |
| EC-3 | `ec3_api_error_delay_thirty_dry_run` | `retry_api_error_test.rs` |
| EC-4 | `ec4_clr_api_error_delay_env_var_accepted` | `retry_api_error_test.rs` |
| EC-5 | `ec5_api_error_delay_cli_wins_over_env` | `retry_api_error_test.rs` |
| EC-6 | `ec6_clr_api_error_delay_invalid_ignored` | `retry_api_error_test.rs` |
| EC-7 | `ec7_api_error_delay_zero_immediate_retry` | `retry_api_error_test.rs` |

---

### EC-1: --help lists --api-error-delay

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--api-error-delay`
- **Exit:** 0
- **Source:** [038_api_error_delay.md](../../../../docs/cli/param/038_api_error_delay.md)
- **Commands:** run, ask

---

### EC-2: --api-error-delay 0 --dry-run → exit 0; zero-second delay accepted

- **Given:** `--api-error-delay 0` and `--dry-run` set
- **When:** `clr --api-error-delay 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced; no error (zero-second delay is a valid value that makes retries fire immediately). **Divergence from EC-3:** value 0 means no sleep between API error retries; value 30 (EC-3) is the default and introduces a 30-second pause per retry attempt
- **Exit:** 0
- **Source:** [038_api_error_delay.md](../../../../docs/cli/param/038_api_error_delay.md)
- **Commands:** run, ask

---

### EC-3: --api-error-delay 30 --dry-run → exit 0; default delay accepted

- **Given:** `--api-error-delay 30` and `--dry-run` set
- **When:** `clr --api-error-delay 30 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced; flag accepted without error (30s is the default delay value)
- **Exit:** 0
- **Source:** [038_api_error_delay.md](../../../../docs/cli/param/038_api_error_delay.md)
- **Commands:** run, ask

---

### EC-4: CLR_API_ERROR_DELAY=5 env var → applied when CLI flag absent

- **Given:** `CLR_API_ERROR_DELAY=5` set; no `--api-error-delay` CLI flag; `--dry-run` set
- **When:** `CLR_API_ERROR_DELAY=5 clr --dry-run "task"`
- **Then:** Exit 0; env var accepted; dry-run output produced
- **Exit:** 0
- **Source:** [038_api_error_delay.md](../../../../docs/cli/param/038_api_error_delay.md)
- **Commands:** run, ask

---

### EC-5: --api-error-delay CLI wins over CLR_API_ERROR_DELAY env var

- **Given:** `CLR_API_ERROR_DELAY=10` set; `--api-error-delay 30` on CLI; `--dry-run` set
- **When:** `CLR_API_ERROR_DELAY=10 clr --api-error-delay 30 --dry-run "task"`
- **Then:** Exit 0; CLI value 30 used (env var 10 ignored); dry-run output produced
- **Exit:** 0
- **Source:** [038_api_error_delay.md](../../../../docs/cli/param/038_api_error_delay.md)
- **Commands:** run, ask

---

### EC-6: CLR_API_ERROR_DELAY=invalid → silently ignored; default 30 used

- **Given:** `CLR_API_ERROR_DELAY=abc` set; no `--api-error-delay` CLI flag; `--dry-run` set
- **When:** `CLR_API_ERROR_DELAY=abc clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored; default 30s delay used when API error retries are triggered (parse failure → stays at default 30)
- **Exit:** 0
- **Source:** [038_api_error_delay.md](../../../../docs/cli/param/038_api_error_delay.md)
- **Commands:** run, ask

---

### EC-7: delay=0 with API error retry → fires immediately; exit 0

- **Given:** fake claude script that emits `"API Error: 500"` on stderr and exits 1 on first invocation, exits 0 on second; `--retry-on-api-error 1 --api-error-delay 0 -p "x"`
- **When:** `clr --retry-on-api-error 1 --api-error-delay 0 -p "x"` using fake script
- **Then:** Exit 0; retry fires without sleep (test completes in < 1s); stderr contains retry message; no delay observed between invocations
- **Exit:** 0
- **Source:** [038_api_error_delay.md](../../../../docs/cli/param/038_api_error_delay.md)
- **Commands:** run, ask

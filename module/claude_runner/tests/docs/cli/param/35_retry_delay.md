# Parameter :: `--retry-delay`

Edge case coverage for the `--retry-delay` parameter. See [035_retry_delay.md](../../../../docs/cli/param/035_retry_delay.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--retry-delay` | Documentation |
| EC-2 | `--retry-delay 0 --dry-run` → exit 0; zero-second delay accepted | Behavioral Divergence |
| EC-3 | `--retry-delay 60 --dry-run` → exit 0; default 60s accepted | Behavioral Divergence |
| EC-4 | `CLR_RETRY_DELAY=30 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_RETRY_DELAY=10 --retry-delay 30 --dry-run` → CLI 30 wins | CLI-wins |
| EC-6 | `CLR_RETRY_DELAY=abc --dry-run` → silently ignored; default 60 used | Validation |
| EC-7 | Fake script exits 2 once then 0; delay=0, retries=1 → retry fires immediately; exit 0 | Integration |

## Test Coverage Summary

- Documentation: 1 test (EC-1)
- Behavioral Divergence: 2 tests (EC-2, EC-3)
- Env Var: 1 test (EC-4)
- CLI-wins: 1 test (EC-5)
- Validation: 1 test (EC-6)
- Integration: 1 test (EC-7)

**Total:** 7 edge cases

## Architectural Constraint

The delay behavior (sleeping N seconds between retry attempts) cannot be exercised in automated
tests at realistic durations — tests use `--retry-delay 0` to make the retry fire immediately.
EC-7 verifies that delay=0 produces a correct immediate retry rather than testing the sleep
duration itself. The default 60s delay is verified at the parse level only (EC-3) since
exercising a 60-second sleep in a test would be impractical. Tests confirm the delay parameter
is accepted and CLI wins over env var; behavioral correctness of the sleep interval is out of
scope for automated testing.

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_retry_delay_help_listed` | `retry_rate_limit_test.rs` |
| EC-2 | `ec2_retry_delay_zero_dry_run` | `retry_rate_limit_test.rs` |
| EC-3 | `ec3_retry_delay_sixty_dry_run` | `retry_rate_limit_test.rs` |
| EC-4 | `ec4_clr_retry_delay_env_var_accepted` | `retry_rate_limit_test.rs` |
| EC-5 | `ec5_retry_delay_cli_wins_over_env` | `retry_rate_limit_test.rs` |
| EC-6 | `ec6_clr_retry_delay_invalid_ignored` | `retry_rate_limit_test.rs` |
| EC-7 | `ec7_retry_delay_zero_immediate_retry` | `retry_rate_limit_test.rs` |

---

### EC-1: --help lists --retry-delay

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--retry-delay`
- **Exit:** 0
- **Source:** [035_retry_delay.md](../../../../docs/cli/param/035_retry_delay.md)
- **Commands:** run, ask

---

### EC-2: --retry-delay 0 --dry-run → exit 0; zero-second delay accepted

- **Given:** `--retry-delay 0` and `--dry-run` set
- **When:** `clr --retry-delay 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced; no error (zero-second delay is a valid value that makes retries fire immediately). **Divergence from EC-3:** value 0 means no sleep between retries; value 60 (EC-3) is the default and introduces a 60-second pause per retry attempt
- **Exit:** 0
- **Source:** [035_retry_delay.md](../../../../docs/cli/param/035_retry_delay.md)
- **Commands:** run, ask

---

### EC-3: --retry-delay 60 --dry-run → exit 0; default delay accepted

- **Given:** `--retry-delay 60` and `--dry-run` set
- **When:** `clr --retry-delay 60 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced; flag accepted without error (60s is the default delay value)
- **Exit:** 0
- **Source:** [035_retry_delay.md](../../../../docs/cli/param/035_retry_delay.md)
- **Commands:** run, ask

---

### EC-4: CLR_RETRY_DELAY=30 env var → applied when CLI flag absent

- **Given:** `CLR_RETRY_DELAY=30` set; no `--retry-delay` CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_DELAY=30 clr --dry-run "task"`
- **Then:** Exit 0; env var accepted; dry-run output produced
- **Exit:** 0
- **Source:** [035_retry_delay.md](../../../../docs/cli/param/035_retry_delay.md)
- **Commands:** run, ask

---

### EC-5: --retry-delay CLI wins over CLR_RETRY_DELAY env var

- **Given:** `CLR_RETRY_DELAY=10` set; `--retry-delay 30` on CLI; `--dry-run` set
- **When:** `CLR_RETRY_DELAY=10 clr --retry-delay 30 --dry-run "task"`
- **Then:** Exit 0; CLI value 30 used (env var 10 ignored); dry-run output produced
- **Exit:** 0
- **Source:** [035_retry_delay.md](../../../../docs/cli/param/035_retry_delay.md)
- **Commands:** run, ask

---

### EC-6: CLR_RETRY_DELAY=invalid → silently ignored; default 60 used

- **Given:** `CLR_RETRY_DELAY=abc` set; no `--retry-delay` CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_DELAY=abc clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored; default 60s delay used when retries are triggered
- **Exit:** 0
- **Source:** [035_retry_delay.md](../../../../docs/cli/param/035_retry_delay.md)
- **Commands:** run, ask

---

### EC-7: delay=0 with rate-limit retry → fires immediately; exit 0

- **Given:** fake claude script that exits 2 on first invocation, exits 0 on second; `--retry-on-rate-limit 1 --retry-delay 0 -p "x"`
- **When:** `clr --retry-on-rate-limit 1 --retry-delay 0 -p "x"` using fake script
- **Then:** Exit 0; retry fires without sleep (test completes in < 1s); stderr contains retry message; no delay observed between invocations
- **Exit:** 0
- **Source:** [035_retry_delay.md](../../../../docs/cli/param/035_retry_delay.md)
- **Commands:** run, ask

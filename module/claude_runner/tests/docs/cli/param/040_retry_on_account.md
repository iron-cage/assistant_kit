# Parameter :: `--retry-on-account`

Edge case coverage for the `--retry-on-account` parameter (new in retry system redesign).
See [040_retry_on_account.md](../../../../docs/cli/param/040_retry_on_account.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--retry-on-account` | Documentation |
| EC-2 | `--retry-on-account 0 --dry-run` → exit 0; explicit zero accepted | Behavioral Divergence |
| EC-3 | `--retry-on-account 2 --dry-run` → exit 0; nonzero accepted | Behavioral Divergence |
| EC-4 | `CLR_RETRY_ON_ACCOUNT=2 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_RETRY_ON_ACCOUNT=1 --retry-on-account 3 --dry-run` → CLI 3 wins | CLI-wins |
| EC-6 | `CLR_RETRY_ON_ACCOUNT=notanumber --dry-run` → silently ignored | Validation |
| EC-7 | Fake emits quota pattern + exits 2 once then 0; retries=1, delay=0 → exit 0; `[Account]` in stderr | Integration |
| EC-8 | Fake always emits quota pattern + exits 2; retries=2 → exit 2; `[Account]` exhaustion in stderr | Integration |

## Test Coverage Summary

- Documentation: 1 test (EC-1)
- Behavioral Divergence: 2 tests (EC-2, EC-3)
- Env Var: 1 test (EC-4)
- CLI-wins: 1 test (EC-5)
- Validation: 1 test (EC-6)
- Integration: 2 tests (EC-7, EC-8)

**Total:** 8 edge cases

## Architectural Constraint

Account class (QuotaExhausted) requires a fake script emitting `"You've hit your limit"` text
and exiting 2. Classification priority ensures QuotaExhausted is detected before the exit-2
RateLimit fallback. `--account-delay 0` is required in integration tests to prevent sleep.

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_retry_on_account_help_listed` | `retry_account_test.rs` |
| EC-2 | `ec2_retry_on_account_zero_dry_run` | `retry_account_test.rs` |
| EC-3 | `ec3_retry_on_account_nonzero_dry_run` | `retry_account_test.rs` |
| EC-4 | `ec4_clr_retry_on_account_env_var_accepted` | `retry_account_test.rs` |
| EC-5 | `ec5_retry_on_account_cli_wins_over_env` | `retry_account_test.rs` |
| EC-6 | `ec6_clr_retry_on_account_invalid_ignored` | `retry_account_test.rs` |
| EC-7 | `ec7_account_retry_succeeds_after_one_quota_exhausted` | `retry_account_test.rs` |
| EC-8 | `ec8_account_retry_exhausted` | `retry_account_test.rs` |

---

### EC-1: --help lists --retry-on-account

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--retry-on-account`
- **Exit:** 0
- **Source:** [040_retry_on_account.md](../../../../docs/cli/param/040_retry_on_account.md)
- **Commands:** run, ask

---

### EC-2: --retry-on-account 0 --dry-run → exit 0; explicit zero accepted

- **Given:** `--retry-on-account 0` and `--dry-run` set
- **When:** `clr --retry-on-account 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced; no retry messages. **Divergence from EC-3:** 0 disables Account retry; 2 enables it
- **Exit:** 0
- **Source:** [040_retry_on_account.md](../../../../docs/cli/param/040_retry_on_account.md)
- **Commands:** run, ask

---

### EC-3: --retry-on-account 2 --dry-run → exit 0; nonzero accepted

- **Given:** `--retry-on-account 2` and `--dry-run` set
- **When:** `clr --retry-on-account 2 --dry-run "task"`
- **Then:** Exit 0; flag accepted without error
- **Exit:** 0
- **Source:** [040_retry_on_account.md](../../../../docs/cli/param/040_retry_on_account.md)
- **Commands:** run, ask

---

### EC-4: CLR_RETRY_ON_ACCOUNT=2 env var → applied when CLI flag absent

- **Given:** `CLR_RETRY_ON_ACCOUNT=2` set; no CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_ON_ACCOUNT=2 clr --dry-run "task"`
- **Then:** Exit 0; env var accepted
- **Exit:** 0
- **Source:** [040_retry_on_account.md](../../../../docs/cli/param/040_retry_on_account.md)
- **Commands:** run, ask

---

### EC-5: CLI wins over CLR_RETRY_ON_ACCOUNT

- **Given:** `CLR_RETRY_ON_ACCOUNT=1` set; `--retry-on-account 3` on CLI; `--dry-run` set
- **When:** `CLR_RETRY_ON_ACCOUNT=1 clr --retry-on-account 3 --dry-run "task"`
- **Then:** Exit 0; CLI value 3 used
- **Exit:** 0
- **Source:** [040_retry_on_account.md](../../../../docs/cli/param/040_retry_on_account.md)
- **Commands:** run, ask

---

### EC-6: CLR_RETRY_ON_ACCOUNT=invalid → silently ignored

- **Given:** `CLR_RETRY_ON_ACCOUNT=notanumber` set; no CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_ON_ACCOUNT=notanumber clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored
- **Exit:** 0
- **Source:** [040_retry_on_account.md](../../../../docs/cli/param/040_retry_on_account.md)
- **Commands:** run, ask

---

### EC-7: Account retry succeeds after one quota-exhausted failure

- **Given:** fake emits `"You've hit your limit"` + exits 2 on first call; exits 0 on second; `--retry-on-account 1 --account-delay 0 -p "x"`
- **When:** `clr --retry-on-account 1 --account-delay 0 --max-sessions 0 -p "x"` using fake
- **Then:** Exit 0; stderr contains `[Account]` retry progress line; two invocations
- **Exit:** 0
- **Source:** [040_retry_on_account.md](../../../../docs/cli/param/040_retry_on_account.md)
- **Commands:** run, ask

---

### EC-8: Account retries exhausted → exit 2; [Account] exhaustion in stderr

- **Given:** fake always emits `"You've hit your limit"` + exits 2; `--retry-on-account 2 --account-delay 0 -p "x"`
- **When:** `clr --retry-on-account 2 --account-delay 0 --max-sessions 0 -p "x"` using fake
- **Then:** Exit 2; stderr contains `[Account]` and "exhausted"; 3 total invocations
- **Exit:** 2
- **Source:** [040_retry_on_account.md](../../../../docs/cli/param/040_retry_on_account.md)
- **Commands:** run, ask

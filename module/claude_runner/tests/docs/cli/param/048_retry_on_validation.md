# Parameter :: `--retry-on-validation`

Edge case coverage for the `--retry-on-validation` parameter (renamed from `--expect-retries`).
See [048_retry_on_validation.md](../../../../docs/cli/param/048_retry_on_validation.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--retry-on-validation` | Documentation |
| EC-2 | `--retry-on-validation 0 --dry-run` → exit 0; explicit zero accepted | Behavioral Divergence |
| EC-3 | `--retry-on-validation 2 --dry-run` → exit 0; nonzero accepted | Behavioral Divergence |
| EC-4 | `CLR_RETRY_ON_VALIDATION=2 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_RETRY_ON_VALIDATION=1 --retry-on-validation 3 --dry-run` → CLI 3 wins | CLI-wins |
| EC-6 | `CLR_RETRY_ON_VALIDATION=notanumber --dry-run` → exit 1; invalid env var rejected | Validation |
| EC-7 | Fake exits 0 with non-matching output once then matching; retries=1, delay=0 → exit 0; `[Validation]` in stderr | Integration |
| EC-8 | Fake always exits 0 with non-matching output; retries=2 → exit 3; `[Validation]` exhaustion in stderr | Integration |
| EC-9 | Old flag `--expect-retries` rejected → exit 1; "unknown option" | Behavioral Divergence |
| EC-10 | No explicit flag; default=auto (fallback 2); fake exits 0 without match once then with match → exit 0 | Integration (Default) |

## Test Coverage Summary

- Documentation: 1 test (EC-1)
- Behavioral Divergence: 3 tests (EC-2, EC-3, EC-9)
- Env Var: 1 test (EC-4)
- CLI-wins: 1 test (EC-5)
- Validation: 1 test (EC-6)
- Integration: 2 tests (EC-7, EC-8)
- Integration (Default): 1 test (EC-10)

**Total:** 10 edge cases

## Architectural Constraint

Validation class requires `--expect "pattern"` to be set. The fake script must exit 0 (success)
but produce output that does NOT match the expect pattern on first invocation, then matching
output on second. `CLR_RETRY_ON_VALIDATION` is the ONLY per-class retry env var that rejects
invalid values (exits 1) rather than silently ignoring them — because validation retry count
must be a valid integer to function. `--validation-delay 0` required in integration tests.
Old flag `--expect-retries` must be confirmed rejected (EC-9).

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_retry_on_validation_help_listed` | `retry_validation_test.rs` |
| EC-2 | `ec2_retry_on_validation_zero_dry_run` | `retry_validation_test.rs` |
| EC-3 | `ec3_retry_on_validation_nonzero_dry_run` | `retry_validation_test.rs` |
| EC-4 | `ec4_clr_retry_on_validation_env_var_accepted` | `retry_validation_test.rs` |
| EC-5 | `ec5_retry_on_validation_cli_wins_over_env` | `retry_validation_test.rs` |
| EC-6 | `ec6_clr_retry_on_validation_invalid_rejected` | `retry_validation_test.rs` |
| EC-7 | `ec7_validation_retry_succeeds_after_one_mismatch` | `retry_validation_test.rs` |
| EC-8 | `ec8_validation_retry_exhausted` | `retry_validation_test.rs` |
| EC-9 | `ec9_old_flag_expect_retries_rejected` | `retry_validation_test.rs` |
| EC-10 | `ec10_validation_fallback_default_fires` | `retry_validation_test.rs` |

---

### EC-1: --help lists --retry-on-validation

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--retry-on-validation`; does NOT contain `--expect-retries`
- **Exit:** 0
- **Source:** [048_retry_on_validation.md](../../../../docs/cli/param/048_retry_on_validation.md)
- **Commands:** run, ask

---

### EC-2: --retry-on-validation 0 --dry-run → exit 0; explicit zero accepted

- **Given:** `--retry-on-validation 0` and `--dry-run` set
- **When:** `clr --retry-on-validation 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced; no retry messages. **Divergence from EC-3:** 0 disables Validation retry; 2 enables it
- **Exit:** 0
- **Source:** [048_retry_on_validation.md](../../../../docs/cli/param/048_retry_on_validation.md)
- **Commands:** run, ask

---

### EC-3: --retry-on-validation 2 --dry-run → exit 0; nonzero accepted

- **Given:** `--retry-on-validation 2` and `--dry-run` set
- **When:** `clr --retry-on-validation 2 --dry-run "task"`
- **Then:** Exit 0; flag accepted without error
- **Exit:** 0
- **Source:** [048_retry_on_validation.md](../../../../docs/cli/param/048_retry_on_validation.md)
- **Commands:** run, ask

---

### EC-4: CLR_RETRY_ON_VALIDATION=2 env var → applied when CLI flag absent

- **Given:** `CLR_RETRY_ON_VALIDATION=2` set; no CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_ON_VALIDATION=2 clr --dry-run "task"`
- **Then:** Exit 0; env var accepted
- **Exit:** 0
- **Source:** [048_retry_on_validation.md](../../../../docs/cli/param/048_retry_on_validation.md)
- **Commands:** run, ask

---

### EC-5: CLI wins over CLR_RETRY_ON_VALIDATION

- **Given:** `CLR_RETRY_ON_VALIDATION=1` set; `--retry-on-validation 3` on CLI; `--dry-run` set
- **When:** `CLR_RETRY_ON_VALIDATION=1 clr --retry-on-validation 3 --dry-run "task"`
- **Then:** Exit 0; CLI value 3 used
- **Exit:** 0
- **Source:** [048_retry_on_validation.md](../../../../docs/cli/param/048_retry_on_validation.md)
- **Commands:** run, ask

---

### EC-6: CLR_RETRY_ON_VALIDATION=invalid → exit 1; invalid env var rejected

- **Given:** `CLR_RETRY_ON_VALIDATION=notanumber` set; no CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_ON_VALIDATION=notanumber clr --dry-run "task"`
- **Then:** Exit 1; stderr indicates invalid value. **Unlike other retry env vars, this one is rejected not silently ignored**
- **Exit:** 1
- **Source:** [048_retry_on_validation.md](../../../../docs/cli/param/048_retry_on_validation.md)
- **Commands:** run, ask

---

### EC-7: Validation retry succeeds after one expect mismatch

- **Given:** fake exits 0 with non-matching output on first call; exits 0 with `"pass"` on second; `--retry-on-validation 1 --validation-delay 0 --expect "pass" -p "x"`
- **When:** `clr --retry-on-validation 1 --validation-delay 0 --expect "pass" --max-sessions 0 -p "x"` using fake
- **Then:** Exit 0; stderr contains `[Validation]` retry progress line; two invocations
- **Exit:** 0
- **Source:** [048_retry_on_validation.md](../../../../docs/cli/param/048_retry_on_validation.md)
- **Commands:** run, ask

---

### EC-8: Validation retries exhausted → exit 3; [Validation] exhaustion in stderr

- **Given:** fake always exits 0 without `"pass"` in output; `--retry-on-validation 2 --validation-delay 0 --expect "pass" -p "x"`
- **When:** `clr --retry-on-validation 2 --validation-delay 0 --expect "pass" --max-sessions 0 -p "x"` using fake
- **Then:** Exit 3; stderr contains `[Validation]` and "exhausted"; 3 total invocations
- **Exit:** 3
- **Source:** [048_retry_on_validation.md](../../../../docs/cli/param/048_retry_on_validation.md)
- **Commands:** run, ask

---

### EC-9: Old flag name --expect-retries rejected at parse time

- **Given:** `--expect-retries 1` (old flag name) passed
- **When:** `clr --expect-retries 1 --dry-run "task"`
- **Then:** Exit 1; stderr contains "unknown option"; parse aborted
- **Exit:** 1
- **Source:** [048_retry_on_validation.md](../../../../docs/cli/param/048_retry_on_validation.md)
- **Commands:** run, ask

---

### EC-10: No explicit --retry-on-validation; fallback default (2) fires for Validation

- **Given:** no `--retry-on-validation` and no `CLR_RETRY_ON_VALIDATION`; fake exits 0 without match once then with match; `--retry-default-delay 0 --expect "pass" -p "x"`
- **When:** `clr --retry-default-delay 0 --expect "pass" --max-sessions 0 -p "x"` using fake
- **Then:** Exit 0; fallback default=2 allows retry; stderr contains `[Validation]` retry message
- **Exit:** 0
- **Source:** [048_retry_on_validation.md](../../../../docs/cli/param/048_retry_on_validation.md)
- **Commands:** run, ask

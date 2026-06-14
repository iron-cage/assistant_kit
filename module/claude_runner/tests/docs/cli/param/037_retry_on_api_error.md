# Parameter :: `--retry-on-api-error`

Edge case coverage for the `--retry-on-api-error` parameter. See [037_retry_on_api_error.md](../../../../docs/cli/param/037_retry_on_api_error.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--retry-on-api-error` | Documentation |
| EC-2 | `--retry-on-api-error 0 --dry-run` → exit 0; explicit no-retry (same as default) | Behavioral Divergence |
| EC-3 | `--retry-on-api-error 2 --dry-run` → exit 0; flag parsed without retry invocation | Behavioral Divergence |
| EC-4 | `CLR_RETRY_ON_API_ERROR=2 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_RETRY_ON_API_ERROR=1 --retry-on-api-error 3 --dry-run` → CLI 3 wins | CLI-wins |
| EC-6 | `CLR_RETRY_ON_API_ERROR=notanumber --dry-run` → silently ignored; default 0 used | Validation |
| EC-7 | Fake exits with `"API Error: 500"` once then 0; retries=1, delay=0 → exit 0; stderr has retry message | Integration |
| EC-8 | Fake always exits with `"API Error: 500"`; retries=2, delay=0 → nonzero exit; stderr has exhaustion message | Integration |
| EC-9 | Fake exits with `"You've hit your limit"` + exit 2; retries=3 → QuotaExhausted not retried | Integration |
| EC-10 | No flag, no env var → default=0; fake exits with API error → immediate exit, no retry | Integration (Default) |

## Test Coverage Summary

- Documentation: 1 test (EC-1)
- Behavioral Divergence: 2 tests (EC-2, EC-3)
- Env Var: 1 test (EC-4)
- CLI-wins: 1 test (EC-5)
- Validation: 1 test (EC-6)
- Integration: 3 tests (EC-7, EC-8, EC-9)
- Integration (Default): 1 test (EC-10)

**Total:** 10 edge cases

## Architectural Constraint

The retry behavior requires a real or fake subprocess that exits with `"API Error: "` text in
stdout or stderr. EC-7 and EC-8 require a fake claude script that emits `"API Error: 500"` on
stderr and exits nonzero. EC-9 requires a fake script that emits `"You've hit your limit"`
(QuotaExhausted pattern) and exits 2 — the classifier must NOT dispatch to the ApiError retry
path because QuotaExhausted has higher priority. Dry-run tests (EC-2 through EC-6) verify
parsing and env-var application only; no subprocess is spawned so no retry logic fires.
The `--api-error-delay 0` flag is used in integration tests to prevent sleep during retry.

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_retry_on_api_error_help_listed` | `retry_api_error_test.rs` |
| EC-2 | `ec2_retry_on_api_error_zero_dry_run` | `retry_api_error_test.rs` |
| EC-3 | `ec3_retry_on_api_error_nonzero_dry_run` | `retry_api_error_test.rs` |
| EC-4 | `ec4_clr_retry_on_api_error_env_var_accepted` | `retry_api_error_test.rs` |
| EC-5 | `ec5_retry_on_api_error_cli_wins_over_env` | `retry_api_error_test.rs` |
| EC-6 | `ec6_clr_retry_on_api_error_invalid_ignored` | `retry_api_error_test.rs` |
| EC-7 | `ec7_retry_succeeds_after_one_api_error` | `retry_api_error_test.rs` |
| EC-8 | `ec8_retry_exhausted_after_all_api_errors` | `retry_api_error_test.rs` |
| EC-9 | `ec9_quota_exhausted_not_retried_as_api_error` | `retry_api_error_test.rs` |
| EC-10 | `ec10_default_no_retry_on_api_error` | `retry_api_error_test.rs` |

---

### EC-1: --help lists --retry-on-api-error

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--retry-on-api-error`
- **Exit:** 0
- **Source:** [037_retry_on_api_error.md](../../../../docs/cli/param/037_retry_on_api_error.md)
- **Commands:** run, ask

---

### EC-2: --retry-on-api-error 0 --dry-run → exit 0; explicit no-retry

- **Given:** `--retry-on-api-error 0` and `--dry-run` set
- **When:** `clr --retry-on-api-error 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced; no retry messages on stderr. **Divergence from EC-3:** value 0 explicitly disables retry (matching default 0); value 2 (EC-3) activates the retry wrapper code path (though in dry-run no subprocess fires)
- **Exit:** 0
- **Source:** [037_retry_on_api_error.md](../../../../docs/cli/param/037_retry_on_api_error.md)
- **Commands:** run, ask

---

### EC-3: --retry-on-api-error 2 --dry-run → exit 0; flag parsed

- **Given:** `--retry-on-api-error 2` and `--dry-run` set
- **When:** `clr --retry-on-api-error 2 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced; no retry messages (subprocess not spawned); flag accepted without error
- **Exit:** 0
- **Source:** [037_retry_on_api_error.md](../../../../docs/cli/param/037_retry_on_api_error.md)
- **Commands:** run, ask

---

### EC-4: CLR_RETRY_ON_API_ERROR=2 env var → applied when CLI flag absent

- **Given:** `CLR_RETRY_ON_API_ERROR=2` set; no `--retry-on-api-error` CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_ON_API_ERROR=2 clr --dry-run "task"`
- **Then:** Exit 0; env var accepted; dry-run output produced (retry logic skipped in dry-run)
- **Exit:** 0
- **Source:** [037_retry_on_api_error.md](../../../../docs/cli/param/037_retry_on_api_error.md)
- **Commands:** run, ask

---

### EC-5: --retry-on-api-error CLI wins over CLR_RETRY_ON_API_ERROR env var

- **Given:** `CLR_RETRY_ON_API_ERROR=1` set; `--retry-on-api-error 3` on CLI; `--dry-run` set
- **When:** `CLR_RETRY_ON_API_ERROR=1 clr --retry-on-api-error 3 --dry-run "task"`
- **Then:** Exit 0; CLI value 3 used (env var 1 ignored); dry-run output produced
- **Exit:** 0
- **Source:** [037_retry_on_api_error.md](../../../../docs/cli/param/037_retry_on_api_error.md)
- **Commands:** run, ask

---

### EC-6: CLR_RETRY_ON_API_ERROR=invalid → silently ignored; default 0 used

- **Given:** `CLR_RETRY_ON_API_ERROR=notanumber` set; no `--retry-on-api-error` CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_ON_API_ERROR=notanumber clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored; default 0 used (no retry configured)
- **Exit:** 0
- **Source:** [037_retry_on_api_error.md](../../../../docs/cli/param/037_retry_on_api_error.md)
- **Commands:** run, ask

---

### EC-7: One API error then success → retried; exit 0

- **Given:** fake claude script that emits `"API Error: 500"` on stderr and exits 1 on first invocation, exits 0 on second; `--retry-on-api-error 1 --api-error-delay 0 -p "x"`
- **When:** `clr --retry-on-api-error 1 --api-error-delay 0 -p "x"` using fake script
- **Then:** Exit 0; stderr contains a retry message; two subprocess invocations observed
- **Exit:** 0
- **Source:** [037_retry_on_api_error.md](../../../../docs/cli/param/037_retry_on_api_error.md)
- **Commands:** run, ask

---

### EC-8: All retries exhausted → nonzero exit; stderr has exhaustion message

- **Given:** fake claude script that always emits `"API Error: 500"` on stderr and exits 1; `--retry-on-api-error 2 --api-error-delay 0 -p "x"`
- **When:** `clr --retry-on-api-error 2 --api-error-delay 0 -p "x"` using always-fail script
- **Then:** Nonzero exit; stderr contains exhaustion message (e.g. "exhausted" or "failed after"); 3 total invocations (1 initial + 2 retries)
- **Exit:** nonzero
- **Source:** [037_retry_on_api_error.md](../../../../docs/cli/param/037_retry_on_api_error.md)
- **Commands:** run, ask

---

### EC-9: QuotaExhausted NOT retried even with --retry-on-api-error set

- **Given:** fake claude script that emits `"You've hit your limit"` on stdout and exits 2; `--retry-on-api-error 3 --api-error-delay 0 -p "x"`
- **When:** `clr --retry-on-api-error 3 --api-error-delay 0 -p "x"` using quota-pattern script
- **Then:** Exit 2 after exactly 1 invocation; no retry messages on stderr; QuotaExhausted classification wins priority over ApiError retry path
- **Exit:** 2
- **Source:** [037_retry_on_api_error.md](../../../../docs/cli/param/037_retry_on_api_error.md)
- **Commands:** run, ask

---

### EC-10: Default retry=0 fires no retry — API error exits immediately

- **Given:** fake claude script that emits `"API Error: 500"` on stderr and exits 1; **no `--retry-on-api-error` flag and no `CLR_RETRY_ON_API_ERROR` env var**; `-p "x"`
- **When:** `clr --max-sessions 0 -p "x"` using fake script (no explicit retry flag)
- **Then:** Nonzero exit; default retry=0 means no retry; stderr contains API error label but no retry message; single subprocess invocation
- **Exit:** nonzero
- **Source:** [037_retry_on_api_error.md](../../../../docs/cli/param/037_retry_on_api_error.md)
- **Commands:** run, ask

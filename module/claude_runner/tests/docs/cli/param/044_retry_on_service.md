# Parameter :: `--retry-on-service`

Edge case coverage for the `--retry-on-service` parameter. See [044_retry_on_service.md](../../../../docs/cli/param/044_retry_on_service.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--retry-on-service` | Documentation |
| EC-2 | `--retry-on-service 0 --dry-run` → exit 0; explicit no-retry | Behavioral Divergence |
| EC-3 | `--retry-on-service 2 --dry-run` → exit 0; flag parsed without retry invocation | Behavioral Divergence |
| EC-4 | `CLR_RETRY_ON_SERVICE=2 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_RETRY_ON_SERVICE=1 --retry-on-service 3 --dry-run` → CLI 3 wins | CLI-wins |
| EC-6 | `CLR_RETRY_ON_SERVICE=notanumber --dry-run` → silently ignored; fallback default used | Validation |
| EC-7 | Fake exits with `"API Error: 500"` once then 0; retries=1, delay=0 → exit 0; stderr has `[Service]` retry message | Integration |
| EC-8 | Fake always exits with `"API Error: 500"`; retries=2, delay=0 → nonzero exit; stderr has `[Service]` exhaustion message | Integration |
| EC-9 | Fake exits with `"You've hit your limit"` + exit 2; retries=3 → Account class; not retried as Service | Integration |
| EC-10 | No flag, no env var → fallback default=2; fake exits API error twice then 0 → exit 0 | Integration (Default) |

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

The retry behavior requires a fake subprocess emitting `"API Error: "` text in stdout or stderr and exiting nonzero. EC-9 requires a fake emitting `"You've hit your limit"` (Account priority over Service). All retry and exhaustion output includes `[Service]` prefix. Old flag name `--retry-on-api-error` must be REJECTED (exit 1, unknown flag) — no backward compatibility.

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_retry_on_service_help_listed` | `retry_service_test.rs` |
| EC-2 | `ec2_retry_on_service_zero_dry_run` | `retry_service_test.rs` |
| EC-3 | `ec3_retry_on_service_nonzero_dry_run` | `retry_service_test.rs` |
| EC-4 | `ec4_clr_retry_on_service_env_var_accepted` | `retry_service_test.rs` |
| EC-5 | `ec5_retry_on_service_cli_wins_over_env` | `retry_service_test.rs` |
| EC-6 | `ec6_clr_retry_on_service_invalid_ignored` | `retry_service_test.rs` |
| EC-7 | `ec7_retry_succeeds_after_one_api_error` | `retry_service_test.rs` |
| EC-8 | `ec8_retry_exhausted_after_all_api_errors` | `retry_service_test.rs` |
| EC-9 | `ec9_quota_exhausted_not_retried_as_service` | `retry_service_test.rs` |
| EC-10 | `ec10_default_retry_fires_on_service_error` | `retry_service_test.rs` |

---

### EC-1: --help lists --retry-on-service

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--retry-on-service`; does NOT contain `--retry-on-api-error`
- **Exit:** 0
- **Source:** [044_retry_on_service.md](../../../../docs/cli/param/044_retry_on_service.md)
- **Commands:** run, ask

---

### EC-2: --retry-on-service 0 --dry-run → exit 0; explicit no-retry

- **Given:** `--retry-on-service 0` and `--dry-run` set
- **When:** `clr --retry-on-service 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced; no retry messages. **Divergence from EC-3:** 0 beats fallback default (2); 2 (EC-3) would activate retry on Service errors
- **Exit:** 0
- **Source:** [044_retry_on_service.md](../../../../docs/cli/param/044_retry_on_service.md)
- **Commands:** run, ask

---

### EC-3: --retry-on-service 2 --dry-run → exit 0; flag parsed

- **Given:** `--retry-on-service 2` and `--dry-run` set
- **When:** `clr --retry-on-service 2 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced; flag accepted without error
- **Exit:** 0
- **Source:** [044_retry_on_service.md](../../../../docs/cli/param/044_retry_on_service.md)
- **Commands:** run, ask

---

### EC-4: CLR_RETRY_ON_SERVICE=2 env var → applied when CLI flag absent

- **Given:** `CLR_RETRY_ON_SERVICE=2` set; no `--retry-on-service` CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_ON_SERVICE=2 clr --dry-run "task"`
- **Then:** Exit 0; env var accepted; dry-run output produced
- **Exit:** 0
- **Source:** [044_retry_on_service.md](../../../../docs/cli/param/044_retry_on_service.md)
- **Commands:** run, ask

---

### EC-5: --retry-on-service CLI wins over CLR_RETRY_ON_SERVICE env var

- **Given:** `CLR_RETRY_ON_SERVICE=1` set; `--retry-on-service 3` on CLI; `--dry-run` set
- **When:** `CLR_RETRY_ON_SERVICE=1 clr --retry-on-service 3 --dry-run "task"`
- **Then:** Exit 0; CLI value 3 used (env var 1 ignored)
- **Exit:** 0
- **Source:** [044_retry_on_service.md](../../../../docs/cli/param/044_retry_on_service.md)
- **Commands:** run, ask

---

### EC-6: CLR_RETRY_ON_SERVICE=invalid → silently ignored; fallback default used

- **Given:** `CLR_RETRY_ON_SERVICE=notanumber` set; no CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_ON_SERVICE=notanumber clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored; fallback default (2) remains configured
- **Exit:** 0
- **Source:** [044_retry_on_service.md](../../../../docs/cli/param/044_retry_on_service.md)
- **Commands:** run, ask

---

### EC-7: One API error then success → retried; exit 0; [Service] prefix

- **Given:** fake claude script emits `"API Error: 500"` on stderr and exits 1 on first invocation, exits 0 on second; `--retry-on-service 1 --service-delay 0 -p "x"`
- **When:** `clr --retry-on-service 1 --service-delay 0 -p "x"` using fake script
- **Then:** Exit 0; stderr contains `[Service]` retry message; two subprocess invocations
- **Exit:** 0
- **Source:** [044_retry_on_service.md](../../../../docs/cli/param/044_retry_on_service.md)
- **Commands:** run, ask

---

### EC-8: All retries exhausted → nonzero exit; [Service] exhaustion message

- **Given:** fake claude script always emits `"API Error: 500"` on stderr and exits 1; `--retry-on-service 2 --service-delay 0 -p "x"`
- **When:** `clr --retry-on-service 2 --service-delay 0 -p "x"` using always-fail script
- **Then:** Nonzero exit; stderr contains `[Service]` exhaustion message; 3 total invocations
- **Exit:** nonzero
- **Source:** [044_retry_on_service.md](../../../../docs/cli/param/044_retry_on_service.md)
- **Commands:** run, ask

---

### EC-9: QuotaExhausted NOT retried even with --retry-on-service set

- **Given:** fake claude script emits `"You've hit your limit"` and exits 2; `--retry-on-service 3 --service-delay 0 -p "x"`
- **When:** `clr --retry-on-service 3 --service-delay 0 -p "x"` using quota-pattern script
- **Then:** Exit 2 after exactly 1 invocation; `[Account]` in stderr; no Service retry fires
- **Exit:** 2
- **Source:** [044_retry_on_service.md](../../../../docs/cli/param/044_retry_on_service.md)
- **Commands:** run, ask

---

### EC-10: Default retry=2 (via fallback) fires on API error

- **Given:** fake claude script emits `"API Error: 500"` and exits 1 twice, then exits 0; no `--retry-on-service` flag and no `CLR_RETRY_ON_SERVICE` env var; `--service-delay 0 --retry-default 2 -p "x"`
- **When:** `clr --service-delay 0 --retry-default 2 -p "x"` using fake script
- **Then:** Exit 0; fallback default=2 fires; stderr has `[Service]` retry messages; three total invocations
- **Exit:** 0
- **Source:** [044_retry_on_service.md](../../../../docs/cli/param/044_retry_on_service.md)
- **Commands:** run, ask

# Parameter :: `--retry-on-rate-limit`

Edge case coverage for the `--retry-on-rate-limit` parameter. See [034_retry_on_rate_limit.md](../../../../docs/cli/param/034_retry_on_rate_limit.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--retry-on-rate-limit` | Documentation |
| EC-2 | `--retry-on-rate-limit 0 --dry-run` → exit 0; explicit disable-retry (overrides default 1) | Behavioral Divergence |
| EC-3 | `--retry-on-rate-limit 3 --dry-run` → exit 0; flag parsed without retry invocation | Behavioral Divergence |
| EC-4 | `CLR_RETRY_ON_RATE_LIMIT=2 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_RETRY_ON_RATE_LIMIT=1 --retry-on-rate-limit 3 --dry-run` → CLI 3 wins | CLI-wins |
| EC-6 | `CLR_RETRY_ON_RATE_LIMIT=notanumber --dry-run` → silently ignored; default 1 used | Validation |
| EC-7 | Fake script exits 2 once then 0; retries=1, delay=0 → exit 0; stderr has retry message | Integration |
| EC-8 | Fake script always exits 2; retries=2, delay=0 → exit 2; stderr has exhaustion message | Integration |
| EC-9 | Fake script exits 2 with `QuotaExhausted` pattern; retries=3 → never retried; exit 2 | Integration |
| EC-10 | No flag, no env var → default=1 fires; fake exits 2 once then 0 → exit 0; stderr has retry | Integration (Default) |

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

The retry behavior (waiting `retry-delay` seconds between attempts) cannot be exercised in tests
without either a real or fake subprocess that exits 2. EC-7 and EC-8 require a fake claude script
that exits with a controlled sequence of codes. EC-9 requires a fake script that emits a
`QuotaExhausted`-pattern string on stderr (e.g. `"You've hit your limit"`) and exits 2 — the
classifier must NOT retry it even though the exit code is 2. Dry-run tests (EC-2 through EC-6)
verify parsing and env-var application only; no subprocess is spawned so no retry logic fires.

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_retry_on_rate_limit_help_listed` | `retry_rate_limit_test.rs` |
| EC-2 | `ec2_retry_on_rate_limit_zero_dry_run` | `retry_rate_limit_test.rs` |
| EC-3 | `ec3_retry_on_rate_limit_nonzero_dry_run` | `retry_rate_limit_test.rs` |
| EC-4 | `ec4_clr_retry_on_rate_limit_env_var_accepted` | `retry_rate_limit_test.rs` |
| EC-5 | `ec5_retry_on_rate_limit_cli_wins_over_env` | `retry_rate_limit_test.rs` |
| EC-6 | `ec6_clr_retry_on_rate_limit_invalid_ignored` | `retry_rate_limit_test.rs` |
| EC-7 | `ec7_retry_succeeds_after_one_rate_limit` | `retry_rate_limit_test.rs` |
| EC-8 | `ec8_retry_exhausted_exits_2` | `retry_rate_limit_test.rs` |
| EC-9 | `ec9_quota_exhausted_not_retried` | `retry_rate_limit_test.rs` |
| EC-10 | `ec10_default_retry_fires_without_flag` | `retry_rate_limit_test.rs` |

---

### EC-1: --help lists --retry-on-rate-limit

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--retry-on-rate-limit`
- **Exit:** 0
- **Source:** [034_retry_on_rate_limit.md](../../../../docs/cli/param/034_retry_on_rate_limit.md)
- **Commands:** run, ask

---

### EC-2: --retry-on-rate-limit 0 --dry-run → exit 0; explicit disable-retry

- **Given:** `--retry-on-rate-limit 0` and `--dry-run` set
- **When:** `clr --retry-on-rate-limit 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced; no retry messages on stderr. **Divergence from EC-3:** value 0 explicitly disables retry (overriding default 1); value 3 (EC-3) activates the retry wrapper code path (though in dry-run it still fires no subprocess)
- **Exit:** 0
- **Source:** [034_retry_on_rate_limit.md](../../../../docs/cli/param/034_retry_on_rate_limit.md)
- **Commands:** run, ask

---

### EC-3: --retry-on-rate-limit 3 --dry-run → exit 0; flag parsed

- **Given:** `--retry-on-rate-limit 3` and `--dry-run` set
- **When:** `clr --retry-on-rate-limit 3 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced; no retry messages (subprocess not spawned); flag accepted without error
- **Exit:** 0
- **Source:** [034_retry_on_rate_limit.md](../../../../docs/cli/param/034_retry_on_rate_limit.md)
- **Commands:** run, ask

---

### EC-4: CLR_RETRY_ON_RATE_LIMIT=2 env var → applied when CLI flag absent

- **Given:** `CLR_RETRY_ON_RATE_LIMIT=2` set; no `--retry-on-rate-limit` CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_ON_RATE_LIMIT=2 clr --dry-run "task"`
- **Then:** Exit 0; env var accepted; dry-run output produced (retry logic skipped in dry-run)
- **Exit:** 0
- **Source:** [034_retry_on_rate_limit.md](../../../../docs/cli/param/034_retry_on_rate_limit.md)
- **Commands:** run, ask

---

### EC-5: --retry-on-rate-limit CLI wins over CLR_RETRY_ON_RATE_LIMIT env var

- **Given:** `CLR_RETRY_ON_RATE_LIMIT=1` set; `--retry-on-rate-limit 3` on CLI; `--dry-run` set
- **When:** `CLR_RETRY_ON_RATE_LIMIT=1 clr --retry-on-rate-limit 3 --dry-run "task"`
- **Then:** Exit 0; CLI value 3 used (env var 1 ignored); dry-run output produced
- **Exit:** 0
- **Source:** [034_retry_on_rate_limit.md](../../../../docs/cli/param/034_retry_on_rate_limit.md)
- **Commands:** run, ask

---

### EC-6: CLR_RETRY_ON_RATE_LIMIT=invalid → silently ignored; default 1 used

- **Given:** `CLR_RETRY_ON_RATE_LIMIT=notanumber` set; no `--retry-on-rate-limit` CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_ON_RATE_LIMIT=notanumber clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored; default 1 used (single retry remains configured)
- **Exit:** 0
- **Source:** [034_retry_on_rate_limit.md](../../../../docs/cli/param/034_retry_on_rate_limit.md)
- **Commands:** run, ask

---

### EC-7: One rate-limit failure then success → retried; exit 0

- **Given:** fake claude script that exits 2 on first invocation, exits 0 on second; `--retry-on-rate-limit 1 --retry-delay 0 -p "x"`
- **When:** `clr --retry-on-rate-limit 1 --retry-delay 0 -p "x"` using fake script
- **Then:** Exit 0; stderr contains a retry message (e.g. "retry" or "retrying"); two subprocess invocations observed
- **Exit:** 0
- **Source:** [034_retry_on_rate_limit.md](../../../../docs/cli/param/034_retry_on_rate_limit.md)
- **Commands:** run, ask

---

### EC-8: All retries exhausted → exit 2; stderr has exhaustion message

- **Given:** fake claude script that always exits 2 (no stdout/stderr patterns matching QuotaExhausted); `--retry-on-rate-limit 2 --retry-delay 0 -p "x"`
- **When:** `clr --retry-on-rate-limit 2 --retry-delay 0 -p "x"` using fake always-fail script
- **Then:** Exit 2; stderr contains exhaustion message (e.g. "exhausted" or "failed after"); 3 total invocations (1 initial + 2 retries)
- **Exit:** 2
- **Source:** [034_retry_on_rate_limit.md](../../../../docs/cli/param/034_retry_on_rate_limit.md)
- **Commands:** run, ask

---

### EC-9: QuotaExhausted exit-2 → not retried even with retries configured

- **Given:** fake claude script that emits `"You've hit your limit"` on stderr and exits 2; `--retry-on-rate-limit 3 --retry-delay 0 -p "x"`
- **When:** `clr --retry-on-rate-limit 3 --retry-delay 0 -p "x"` using quota-pattern fake script
- **Then:** Exit 2 after exactly 1 invocation; no retry messages on stderr (QuotaExhausted is non-transient and must never be retried); only 1 subprocess spawned
- **Exit:** 2
- **Source:** [034_retry_on_rate_limit.md](../../../../docs/cli/param/034_retry_on_rate_limit.md)
- **Commands:** run, ask

---

### EC-10: Default retry=1 fires without explicit flag — one rate-limit exit then success

- **Given:** fake claude script that exits 2 on first invocation, exits 0 on second; **no `--retry-on-rate-limit` flag and no `CLR_RETRY_ON_RATE_LIMIT` env var**; `--retry-delay 0` to prevent slow test; `-p "x"`
- **When:** `clr --retry-delay 0 -p "x"` using fake script (no explicit retry flag)
- **Then:** Exit 0; default retry=1 fires; stderr contains a retry message; two subprocess invocations observed
- **Exit:** 0
- **Source:** [034_retry_on_rate_limit.md](../../../../docs/cli/param/034_retry_on_rate_limit.md)
- **Commands:** run, ask

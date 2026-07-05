# Parameter :: `--retry-on-transient`

Edge case coverage for the `--retry-on-transient` parameter (renamed from `--retry-on-rate-limit`).
See [034_retry_on_transient.md](../../../../docs/cli/param/034_retry_on_transient.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--retry-on-transient` | Documentation |
| EC-2 | `--retry-on-transient 0 --dry-run` → exit 0; explicit disable accepted | Behavioral Divergence |
| EC-3 | `--retry-on-transient 3 --dry-run` → exit 0; nonzero value accepted | Behavioral Divergence |
| EC-4 | `CLR_RETRY_ON_TRANSIENT=2 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_RETRY_ON_TRANSIENT=1 --retry-on-transient 3 --dry-run` → CLI 3 wins | CLI-wins |
| EC-6 | `CLR_RETRY_ON_TRANSIENT=notanumber --dry-run` → silently ignored | Validation |
| EC-7 | Fake exits 2 once then 0; retries=1, delay=0 → exit 0; stderr has `[Transient]` retry message | Integration |
| EC-8 | Fake always exits 2; retries=2, delay=0 → exit 2; stderr has exhaustion with `[Transient]` | Integration |
| EC-9 | Old flag `--retry-on-rate-limit` rejected → exit 1; error message contains "unknown option" | Behavioral Divergence |
| EC-10 | No explicit flag; fallback default=2 fires; fake exits 2 once then 0 → exit 0 | Integration (Default) |

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

The retry behavior requires a fake subprocess that exits 2 (no QuotaExhausted pattern). EC-7 and EC-8
use fake claude scripts with controlled exit sequences. `--transient-delay 0` (or `--retry-default-delay 0`)
is required in integration tests to prevent sleep. The old flag name `--retry-on-rate-limit` must be
confirmed rejected at parse time (EC-9) — a test using the old name must fail with "unknown option".

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_retry_on_transient_help_listed` | `retry_transient_test.rs` |
| EC-2 | `ec2_retry_on_transient_zero_dry_run` | `retry_transient_test.rs` |
| EC-3 | `ec3_retry_on_transient_nonzero_dry_run` | `retry_transient_test.rs` |
| EC-4 | `ec4_clr_retry_on_transient_env_var_accepted` | `retry_transient_test.rs` |
| EC-5 | `ec5_retry_on_transient_cli_wins_over_env` | `retry_transient_test.rs` |
| EC-6 | `ec6_clr_retry_on_transient_invalid_ignored` | `retry_transient_test.rs` |
| EC-7 | `ec7_transient_retry_succeeds_after_one_failure` | `retry_transient_test.rs` |
| EC-8 | `ec8_transient_retry_exhausted_exits_2` | `retry_transient_test.rs` |
| EC-9 | `ec9_old_flag_name_rejected` | `retry_transient_test.rs` |
| EC-10 | `ec10_transient_fallback_default_fires_without_flag` | `retry_transient_test.rs` |

---

### EC-1: --help lists --retry-on-transient

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--retry-on-transient`; does NOT contain `--retry-on-rate-limit`
- **Exit:** 0
- **Source:** [034_retry_on_transient.md](../../../../docs/cli/param/034_retry_on_transient.md)
- **Commands:** run, ask

---

### EC-2: --retry-on-transient 0 --dry-run → exit 0; explicit disable accepted

- **Given:** `--retry-on-transient 0` and `--dry-run` set
- **When:** `clr --retry-on-transient 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced; no retry messages on stderr. **Divergence from EC-3:** value 0 explicitly disables Transient retry; value 3 activates retry code path (no subprocess in dry-run)
- **Exit:** 0
- **Source:** [034_retry_on_transient.md](../../../../docs/cli/param/034_retry_on_transient.md)
- **Commands:** run, ask

---

### EC-3: --retry-on-transient 3 --dry-run → exit 0; flag accepted

- **Given:** `--retry-on-transient 3` and `--dry-run` set
- **When:** `clr --retry-on-transient 3 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced; flag accepted without error
- **Exit:** 0
- **Source:** [034_retry_on_transient.md](../../../../docs/cli/param/034_retry_on_transient.md)
- **Commands:** run, ask

---

### EC-4: CLR_RETRY_ON_TRANSIENT=2 env var → applied when CLI flag absent

- **Given:** `CLR_RETRY_ON_TRANSIENT=2` set; no `--retry-on-transient` CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_ON_TRANSIENT=2 clr --dry-run "task"`
- **Then:** Exit 0; env var accepted; dry-run output produced
- **Exit:** 0
- **Source:** [034_retry_on_transient.md](../../../../docs/cli/param/034_retry_on_transient.md)
- **Commands:** run, ask

---

### EC-5: --retry-on-transient CLI wins over CLR_RETRY_ON_TRANSIENT

- **Given:** `CLR_RETRY_ON_TRANSIENT=1` set; `--retry-on-transient 3` on CLI; `--dry-run` set
- **When:** `CLR_RETRY_ON_TRANSIENT=1 clr --retry-on-transient 3 --dry-run "task"`
- **Then:** Exit 0; CLI value 3 used; dry-run output produced
- **Exit:** 0
- **Source:** [034_retry_on_transient.md](../../../../docs/cli/param/034_retry_on_transient.md)
- **Commands:** run, ask

---

### EC-6: CLR_RETRY_ON_TRANSIENT=invalid → silently ignored

- **Given:** `CLR_RETRY_ON_TRANSIENT=notanumber` set; no CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_ON_TRANSIENT=notanumber clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored; fallback default applies
- **Exit:** 0
- **Source:** [034_retry_on_transient.md](../../../../docs/cli/param/034_retry_on_transient.md)
- **Commands:** run, ask

---

### EC-7: One Transient failure then success → retried; exit 0; stderr has [Transient] tag

- **Given:** fake claude exits 2 on first call (no pattern text), exits 0 on second; `--retry-on-transient 1 --transient-delay 0 -p "x"`
- **When:** `clr --retry-on-transient 1 --transient-delay 0 --max-sessions 0 -p "x"` using fake
- **Then:** Exit 0; stderr contains `[Transient]` retry progress line; two subprocess invocations
- **Exit:** 0
- **Source:** [034_retry_on_transient.md](../../../../docs/cli/param/034_retry_on_transient.md)
- **Commands:** run, ask

---

### EC-8: All Transient retries exhausted → exit 2; stderr has [Transient] exhaustion

- **Given:** fake claude always exits 2 (no pattern); `--retry-on-transient 2 --transient-delay 0 -p "x"`
- **When:** `clr --retry-on-transient 2 --transient-delay 0 --max-sessions 0 -p "x"` using fake
- **Then:** Exit 2; stderr contains `[Transient]` and "exhausted"; 3 total invocations (1 + 2 retries)
- **Exit:** 2
- **Source:** [034_retry_on_transient.md](../../../../docs/cli/param/034_retry_on_transient.md)
- **Commands:** run, ask

---

### EC-9: Old flag name --retry-on-rate-limit rejected at parse time

- **Given:** `--retry-on-rate-limit 1` (old flag name) passed
- **When:** `clr --retry-on-rate-limit 1 --dry-run "task"`
- **Then:** Exit 1; stderr contains "unknown option"; parse aborted before execution
- **Exit:** 1
- **Source:** [034_retry_on_transient.md](../../../../docs/cli/param/034_retry_on_transient.md)
- **Commands:** run, ask

---

### EC-10: No explicit flag; fallback default (2) fires for Transient

- **Given:** no `--retry-on-transient` and no `CLR_RETRY_ON_TRANSIENT`; fake exits 2 once then 0; `--retry-default-delay 0 -p "x"`
- **When:** `clr --retry-default-delay 0 --max-sessions 0 -p "x"` using fake script
- **Then:** Exit 0; fallback default=2 allows at least 1 retry; stderr contains `[Transient]` retry message
- **Exit:** 0
- **Source:** [034_retry_on_transient.md](../../../../docs/cli/param/034_retry_on_transient.md)
- **Commands:** run, ask

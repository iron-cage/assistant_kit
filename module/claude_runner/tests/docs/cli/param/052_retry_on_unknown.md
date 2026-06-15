# Parameter :: `--retry-on-unknown`

Edge case coverage for the `--retry-on-unknown` parameter (renamed from `--retry-on-unknown-error`).
See [052_retry_on_unknown.md](../../../../docs/cli/param/052_retry_on_unknown.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--retry-on-unknown` | Documentation |
| EC-2 | `--retry-on-unknown 0 --dry-run` → exit 0; explicit zero accepted | Behavioral Divergence |
| EC-3 | `--retry-on-unknown 2 --dry-run` → exit 0; nonzero accepted | Behavioral Divergence |
| EC-4 | `CLR_RETRY_ON_UNKNOWN=2 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_RETRY_ON_UNKNOWN=1 --retry-on-unknown 3 --dry-run` → CLI 3 wins | CLI-wins |
| EC-6 | `CLR_RETRY_ON_UNKNOWN=notanumber --dry-run` → silently ignored | Validation |
| EC-7 | Fake emits unrecognized output + exits 5 once then 0; retries=1, delay=0 → exit 0; `[Unknown]` in stderr | Integration |
| EC-8 | Fake always emits unrecognized output + exits 5; retries=2 → exit 5; `[Unknown]` exhaustion in stderr | Integration |
| EC-9 | Old flag `--retry-on-unknown-error` rejected → exit 1; "unknown option" | Behavioral Divergence |
| EC-10 | No explicit flag; default=auto (fallback 2); fake exits 5 once then 0 → exit 0 | Integration (Default) |

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

Unknown class is the catch-all: any subprocess exit with no recognized pattern and exit code
that is not 2 (Transient) or 4 (Process) classifies as Unknown. The fake script emits
`"something went wrong"` and exits 5 — no recognized text pattern, no special exit code →
Unknown. `--unknown-delay 0` required in integration tests. Old flag
`--retry-on-unknown-error` must be confirmed rejected (EC-9).

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_retry_on_unknown_help_listed` | `retry_unknown_test.rs` |
| EC-2 | `ec2_retry_on_unknown_zero_dry_run` | `retry_unknown_test.rs` |
| EC-3 | `ec3_retry_on_unknown_nonzero_dry_run` | `retry_unknown_test.rs` |
| EC-4 | `ec4_clr_retry_on_unknown_env_var_accepted` | `retry_unknown_test.rs` |
| EC-5 | `ec5_retry_on_unknown_cli_wins_over_env` | `retry_unknown_test.rs` |
| EC-6 | `ec6_clr_retry_on_unknown_invalid_ignored` | `retry_unknown_test.rs` |
| EC-7 | `ec7_unknown_retry_succeeds_after_one_failure` | `retry_unknown_test.rs` |
| EC-8 | `ec8_unknown_retry_exhausted` | `retry_unknown_test.rs` |
| EC-9 | `ec9_old_flag_retry_on_unknown_error_rejected` | `retry_unknown_test.rs` |
| EC-10 | `ec10_unknown_fallback_default_fires` | `retry_unknown_test.rs` |

---

### EC-1: --help lists --retry-on-unknown

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--retry-on-unknown`; does NOT contain `--retry-on-unknown-error`
- **Exit:** 0
- **Source:** [052_retry_on_unknown.md](../../../../docs/cli/param/052_retry_on_unknown.md)
- **Commands:** run, ask

---

### EC-2: --retry-on-unknown 0 --dry-run → exit 0; explicit zero accepted

- **Given:** `--retry-on-unknown 0` and `--dry-run` set
- **When:** `clr --retry-on-unknown 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced; no retry messages. **Divergence from EC-3:** 0 disables Unknown retry; 2 enables it
- **Exit:** 0
- **Source:** [052_retry_on_unknown.md](../../../../docs/cli/param/052_retry_on_unknown.md)
- **Commands:** run, ask

---

### EC-3: --retry-on-unknown 2 --dry-run → exit 0; nonzero accepted

- **Given:** `--retry-on-unknown 2` and `--dry-run` set
- **When:** `clr --retry-on-unknown 2 --dry-run "task"`
- **Then:** Exit 0; flag accepted without error
- **Exit:** 0
- **Source:** [052_retry_on_unknown.md](../../../../docs/cli/param/052_retry_on_unknown.md)
- **Commands:** run, ask

---

### EC-4: CLR_RETRY_ON_UNKNOWN=2 env var → applied when CLI flag absent

- **Given:** `CLR_RETRY_ON_UNKNOWN=2` set; no CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_ON_UNKNOWN=2 clr --dry-run "task"`
- **Then:** Exit 0; env var accepted
- **Exit:** 0
- **Source:** [052_retry_on_unknown.md](../../../../docs/cli/param/052_retry_on_unknown.md)
- **Commands:** run, ask

---

### EC-5: CLI wins over CLR_RETRY_ON_UNKNOWN

- **Given:** `CLR_RETRY_ON_UNKNOWN=1` set; `--retry-on-unknown 3` on CLI; `--dry-run` set
- **When:** `CLR_RETRY_ON_UNKNOWN=1 clr --retry-on-unknown 3 --dry-run "task"`
- **Then:** Exit 0; CLI value 3 used
- **Exit:** 0
- **Source:** [052_retry_on_unknown.md](../../../../docs/cli/param/052_retry_on_unknown.md)
- **Commands:** run, ask

---

### EC-6: CLR_RETRY_ON_UNKNOWN=invalid → silently ignored

- **Given:** `CLR_RETRY_ON_UNKNOWN=notanumber` set; no CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_ON_UNKNOWN=notanumber clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored
- **Exit:** 0
- **Source:** [052_retry_on_unknown.md](../../../../docs/cli/param/052_retry_on_unknown.md)
- **Commands:** run, ask

---

### EC-7: Unknown retry succeeds after one unrecognized failure

- **Given:** fake emits `"something went wrong"` + exits 5 on first call; exits 0 on second; `--retry-on-unknown 1 --unknown-delay 0 -p "x"`
- **When:** `clr --retry-on-unknown 1 --unknown-delay 0 --max-sessions 0 -p "x"` using fake
- **Then:** Exit 0; stderr contains `[Unknown]` retry progress line; two invocations
- **Exit:** 0
- **Source:** [052_retry_on_unknown.md](../../../../docs/cli/param/052_retry_on_unknown.md)
- **Commands:** run, ask

---

### EC-8: Unknown retries exhausted → exit 5; [Unknown] exhaustion in stderr

- **Given:** fake always emits `"something went wrong"` + exits 5; `--retry-on-unknown 2 --unknown-delay 0 -p "x"`
- **When:** `clr --retry-on-unknown 2 --unknown-delay 0 --max-sessions 0 -p "x"` using fake
- **Then:** Exit 5; stderr contains `[Unknown]` and "exhausted"; 3 total invocations
- **Exit:** 5
- **Source:** [052_retry_on_unknown.md](../../../../docs/cli/param/052_retry_on_unknown.md)
- **Commands:** run, ask

---

### EC-9: Old flag name --retry-on-unknown-error rejected at parse time

- **Given:** `--retry-on-unknown-error 1` (old flag name) passed
- **When:** `clr --retry-on-unknown-error 1 --dry-run "task"`
- **Then:** Exit 1; stderr contains "unknown option"; parse aborted
- **Exit:** 1
- **Source:** [052_retry_on_unknown.md](../../../../docs/cli/param/052_retry_on_unknown.md)
- **Commands:** run, ask

---

### EC-10: No explicit --retry-on-unknown; fallback default (2) fires for Unknown

- **Given:** no `--retry-on-unknown` and no `CLR_RETRY_ON_UNKNOWN`; fake exits 5 once then 0; `--retry-default-delay 0 -p "x"`
- **When:** `clr --retry-default-delay 0 --max-sessions 0 -p "x"` using fake
- **Then:** Exit 0; fallback default=2 allows retry; stderr contains `[Unknown]` retry message
- **Exit:** 0
- **Source:** [052_retry_on_unknown.md](../../../../docs/cli/param/052_retry_on_unknown.md)
- **Commands:** run, ask

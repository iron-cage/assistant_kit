# Parameter :: `--retry-on-unknown-error`

Edge case coverage for the `--retry-on-unknown-error` parameter. See [039_retry_on_unknown_error.md](../../../../docs/cli/param/039_retry_on_unknown_error.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--retry-on-unknown-error` | Documentation |
| EC-2 | `--retry-on-unknown-error 0 --dry-run` → exit 0; explicit no-retry (same as default) | Behavioral Divergence |
| EC-3 | `--retry-on-unknown-error 1 --dry-run` → exit 0; flag parsed without retry invocation | Behavioral Divergence |
| EC-4 | `CLR_RETRY_ON_UNKNOWN_ERROR=1 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_RETRY_ON_UNKNOWN_ERROR=1 --retry-on-unknown-error 2 --dry-run` → CLI 2 wins | CLI-wins |
| EC-6 | `CLR_RETRY_ON_UNKNOWN_ERROR=bad --dry-run` → silently ignored; default 0 used | Validation |
| EC-7 | Fake exits nonzero (no pattern) once then 0; retries=1, delay=0 → exit 0; stderr has retry message | Integration |
| EC-8 | Fake always exits nonzero (no pattern); retries=1, delay=0 → nonzero exit; stderr has exhaustion message | Integration |
| EC-9 | No flag, no env var → default=0; fake exits nonzero (no pattern) → immediate exit, no retry | Integration (Default) |

## Test Coverage Summary

- Documentation: 1 test (EC-1)
- Behavioral Divergence: 2 tests (EC-2, EC-3)
- Env Var: 1 test (EC-4)
- CLI-wins: 1 test (EC-5)
- Validation: 1 test (EC-6)
- Integration: 2 tests (EC-7, EC-8)
- Integration (Default): 1 test (EC-9)

**Total:** 9 edge cases

## Architectural Constraint

The retry behavior requires a real or fake subprocess that exits nonzero with no recognized error
pattern text (no `"API Error: "`, no `"You've hit your limit"`, no auth text). This triggers the
`ErrorKind::Unknown` classification. EC-7 and EC-8 require a fake claude script that exits with a
code like 42 (not 0, not 2, not > 128) and empty stderr/stdout — ensuring the `Unknown` fallback
path is taken. Dry-run tests (EC-2 through EC-6) verify parsing and env-var application only; no
subprocess is spawned so no retry logic fires. Unknown retries use `--retry-delay` for the cooldown
between attempts (shared with rate-limit retries); EC-7 and EC-8 specify `--retry-delay 0` to
prevent 30-second sleeps during automated testing.

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_retry_on_unknown_error_help_listed` | `retry_unknown_error_test.rs` |
| EC-2 | `ec2_retry_on_unknown_error_zero_dry_run` | `retry_unknown_error_test.rs` |
| EC-3 | `ec3_retry_on_unknown_error_nonzero_dry_run` | `retry_unknown_error_test.rs` |
| EC-4 | `ec4_clr_retry_on_unknown_error_env_var_accepted` | `retry_unknown_error_test.rs` |
| EC-5 | `ec5_retry_on_unknown_error_cli_wins_over_env` | `retry_unknown_error_test.rs` |
| EC-6 | `ec6_clr_retry_on_unknown_error_invalid_ignored` | `retry_unknown_error_test.rs` |
| EC-7 | `ec7_retry_succeeds_after_one_unknown_error` | `retry_unknown_error_test.rs` |
| EC-8 | `ec8_retry_exhausted_after_all_unknown_errors` | `retry_unknown_error_test.rs` |
| EC-9 | `ec9_default_no_retry_on_unknown_error` | `retry_unknown_error_test.rs` |

---

### EC-1: --help lists --retry-on-unknown-error

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--retry-on-unknown-error`
- **Exit:** 0
- **Source:** [039_retry_on_unknown_error.md](../../../../docs/cli/param/039_retry_on_unknown_error.md)
- **Commands:** run, ask

---

### EC-2: --retry-on-unknown-error 0 --dry-run → exit 0; explicit no-retry

- **Given:** `--retry-on-unknown-error 0` and `--dry-run` set
- **When:** `clr --retry-on-unknown-error 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced; no retry messages on stderr. **Divergence from EC-3:** value 0 explicitly disables retry (matching default 0); value 1 (EC-3) activates the retry wrapper code path (though in dry-run no subprocess fires)
- **Exit:** 0
- **Source:** [039_retry_on_unknown_error.md](../../../../docs/cli/param/039_retry_on_unknown_error.md)
- **Commands:** run, ask

---

### EC-3: --retry-on-unknown-error 1 --dry-run → exit 0; flag parsed

- **Given:** `--retry-on-unknown-error 1` and `--dry-run` set
- **When:** `clr --retry-on-unknown-error 1 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced; no retry messages (subprocess not spawned); flag accepted without error
- **Exit:** 0
- **Source:** [039_retry_on_unknown_error.md](../../../../docs/cli/param/039_retry_on_unknown_error.md)
- **Commands:** run, ask

---

### EC-4: CLR_RETRY_ON_UNKNOWN_ERROR=1 env var → applied when CLI flag absent

- **Given:** `CLR_RETRY_ON_UNKNOWN_ERROR=1` set; no `--retry-on-unknown-error` CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_ON_UNKNOWN_ERROR=1 clr --dry-run "task"`
- **Then:** Exit 0; env var accepted; dry-run output produced (retry logic skipped in dry-run)
- **Exit:** 0
- **Source:** [039_retry_on_unknown_error.md](../../../../docs/cli/param/039_retry_on_unknown_error.md)
- **Commands:** run, ask

---

### EC-5: --retry-on-unknown-error CLI wins over CLR_RETRY_ON_UNKNOWN_ERROR env var

- **Given:** `CLR_RETRY_ON_UNKNOWN_ERROR=1` set; `--retry-on-unknown-error 2` on CLI; `--dry-run` set
- **When:** `CLR_RETRY_ON_UNKNOWN_ERROR=1 clr --retry-on-unknown-error 2 --dry-run "task"`
- **Then:** Exit 0; CLI value 2 used (env var 1 ignored); dry-run output produced
- **Exit:** 0
- **Source:** [039_retry_on_unknown_error.md](../../../../docs/cli/param/039_retry_on_unknown_error.md)
- **Commands:** run, ask

---

### EC-6: CLR_RETRY_ON_UNKNOWN_ERROR=invalid → silently ignored; default 0 used

- **Given:** `CLR_RETRY_ON_UNKNOWN_ERROR=bad` set; no `--retry-on-unknown-error` CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_ON_UNKNOWN_ERROR=bad clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored; default 0 used (no retry configured)
- **Exit:** 0
- **Source:** [039_retry_on_unknown_error.md](../../../../docs/cli/param/039_retry_on_unknown_error.md)
- **Commands:** run, ask

---

### EC-7: One unknown error then success → retried; exit 0

- **Given:** fake claude script that exits 42 (no pattern text) on first invocation, exits 0 on second; `--retry-on-unknown-error 1 --retry-delay 0 -p "x"`
- **When:** `clr --retry-on-unknown-error 1 --retry-delay 0 -p "x"` using fake script
- **Then:** Exit 0; stderr contains a retry message; two subprocess invocations observed; retry fires immediately (delay=0)
- **Exit:** 0
- **Source:** [039_retry_on_unknown_error.md](../../../../docs/cli/param/039_retry_on_unknown_error.md)
- **Commands:** run, ask

---

### EC-8: All retries exhausted → nonzero exit; stderr has exhaustion message

- **Given:** fake claude script that always exits 42 (no pattern text); `--retry-on-unknown-error 1 --retry-delay 0 -p "x"`
- **When:** `clr --retry-on-unknown-error 1 --retry-delay 0 -p "x"` using always-fail script
- **Then:** Nonzero exit; stderr contains exhaustion message (e.g. "exhausted" or "failed after"); 2 total invocations (1 initial + 1 retry); retry fires immediately (delay=0)
- **Exit:** nonzero
- **Source:** [039_retry_on_unknown_error.md](../../../../docs/cli/param/039_retry_on_unknown_error.md)
- **Commands:** run, ask

---

### EC-9: Default retry=0 fires no retry — unknown error exits immediately

- **Given:** fake claude script that exits 42 (no pattern text); **no `--retry-on-unknown-error` flag and no `CLR_RETRY_ON_UNKNOWN_ERROR` env var**; `-p "x"`
- **When:** `clr --max-sessions 0 -p "x"` using fake script (no explicit retry flag)
- **Then:** Nonzero exit; default retry=0 means no retry; stderr contains unknown error label but no retry message; single subprocess invocation
- **Exit:** nonzero
- **Source:** [039_retry_on_unknown_error.md](../../../../docs/cli/param/039_retry_on_unknown_error.md)
- **Commands:** run, ask

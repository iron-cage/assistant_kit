# Parameter :: `--retry-on-auth`

Edge case coverage for the `--retry-on-auth` parameter (new in retry system redesign).
See [042_retry_on_auth.md](../../../../docs/cli/param/042_retry_on_auth.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--help` output contains `--retry-on-auth` | Documentation |
| EC-2 | `--retry-on-auth 0 --dry-run` → exit 0; explicit zero accepted | Behavioral Divergence |
| EC-3 | `--retry-on-auth 2 --dry-run` → exit 0; nonzero accepted | Behavioral Divergence |
| EC-4 | `CLR_RETRY_ON_AUTH=2 --dry-run` → exit 0; env var applied | Env Var |
| EC-5 | `CLR_RETRY_ON_AUTH=1 --retry-on-auth 3 --dry-run` → CLI 3 wins | CLI-wins |
| EC-6 | `CLR_RETRY_ON_AUTH=notanumber --dry-run` → silently ignored | Validation |
| EC-7 | Fake emits auth pattern + exits 1; retries=1, delay=0 → exit 1 immediately (fail-fast, no retry); `[Auth]` in stderr; invocation count=1 | Integration (BUG-315) |
| EC-8 | Fake always emits auth pattern + exits 1; retries=2, delay=0 → exit 1 immediately (fail-fast, no retry, no "exhausted"); invocation count=1 | Integration (BUG-315) |

## Test Coverage Summary

- Documentation: 1 test (EC-1)
- Behavioral Divergence: 2 tests (EC-2, EC-3)
- Env Var: 1 test (EC-4)
- CLI-wins: 1 test (EC-5)
- Validation: 1 test (EC-6)
- Integration (BUG-315): 2 tests (EC-7, EC-8)

**Total:** 8 edge cases

## Architectural Constraint

Auth class requires a fake script emitting `"Your organization does not have access to Claude"` on
stdout or stderr and exiting nonzero. `--auth-delay 0` is required in integration tests.

## Implementation Notes

| EC | Test Function | File |
|----|---------------|------|
| EC-1 | `ec1_retry_on_auth_help_listed` | `retry_auth_test.rs` |
| EC-2 | `ec2_retry_on_auth_zero_dry_run` | `retry_auth_test.rs` |
| EC-3 | `ec3_retry_on_auth_nonzero_dry_run` | `retry_auth_test.rs` |
| EC-4 | `ec4_clr_retry_on_auth_env_var_accepted` | `retry_auth_test.rs` |
| EC-5 | `ec5_retry_on_auth_cli_wins_over_env` | `retry_auth_test.rs` |
| EC-6 | `ec6_clr_retry_on_auth_invalid_ignored` | `retry_auth_test.rs` |
| EC-7 | `ec7_auth_error_exits_immediately_without_retry` | `retry_auth_test.rs` |
| EC-8 | `ec8_auth_error_exits_immediately_regardless_of_retry_budget` | `retry_auth_test.rs` |

---

### EC-1: --help lists --retry-on-auth

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--retry-on-auth`
- **Exit:** 0
- **Source:** [042_retry_on_auth.md](../../../../docs/cli/param/042_retry_on_auth.md)
- **Commands:** run, ask

---

### EC-2: --retry-on-auth 0 --dry-run → exit 0; explicit zero accepted

- **Given:** `--retry-on-auth 0` and `--dry-run` set
- **When:** `clr --retry-on-auth 0 --dry-run "task"`
- **Then:** Exit 0; dry-run output produced; no retry messages. **Divergence from EC-3:** 0 disables Auth retry; 2 enables it
- **Exit:** 0
- **Source:** [042_retry_on_auth.md](../../../../docs/cli/param/042_retry_on_auth.md)
- **Commands:** run, ask

---

### EC-3: --retry-on-auth 2 --dry-run → exit 0; nonzero accepted

- **Given:** `--retry-on-auth 2` and `--dry-run` set
- **When:** `clr --retry-on-auth 2 --dry-run "task"`
- **Then:** Exit 0; flag accepted without error
- **Exit:** 0
- **Source:** [042_retry_on_auth.md](../../../../docs/cli/param/042_retry_on_auth.md)
- **Commands:** run, ask

---

### EC-4: CLR_RETRY_ON_AUTH=2 env var → applied when CLI flag absent

- **Given:** `CLR_RETRY_ON_AUTH=2` set; no CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_ON_AUTH=2 clr --dry-run "task"`
- **Then:** Exit 0; env var accepted
- **Exit:** 0
- **Source:** [042_retry_on_auth.md](../../../../docs/cli/param/042_retry_on_auth.md)
- **Commands:** run, ask

---

### EC-5: CLI wins over CLR_RETRY_ON_AUTH

- **Given:** `CLR_RETRY_ON_AUTH=1` set; `--retry-on-auth 3` on CLI; `--dry-run` set
- **When:** `CLR_RETRY_ON_AUTH=1 clr --retry-on-auth 3 --dry-run "task"`
- **Then:** Exit 0; CLI value 3 used
- **Exit:** 0
- **Source:** [042_retry_on_auth.md](../../../../docs/cli/param/042_retry_on_auth.md)
- **Commands:** run, ask

---

### EC-6: CLR_RETRY_ON_AUTH=invalid → silently ignored

- **Given:** `CLR_RETRY_ON_AUTH=notanumber` set; no CLI flag; `--dry-run` set
- **When:** `CLR_RETRY_ON_AUTH=notanumber clr --dry-run "task"`
- **Then:** Exit 0; invalid env var silently ignored
- **Exit:** 0
- **Source:** [042_retry_on_auth.md](../../../../docs/cli/param/042_retry_on_auth.md)
- **Commands:** run, ask

---

### EC-7: Auth error exits immediately even with `--retry-on-auth 1` (fail-fast; BUG-315)

- **Given:** fake emits auth pattern + exits 1 (would exit 0 on 2nd call, but 2nd call never fires); `--retry-on-auth 1 --auth-delay 0 -p "x"`
- **When:** `clr --retry-on-auth 1 --auth-delay 0 --max-sessions 0 -p "x"` using fake
- **Then:** Exit 1 immediately; stderr contains `[Auth]`; invocation count = 1 (no retry); no "retry" or "retrying" in stderr
- **Exit:** 1
- **Note:** Fix(BUG-315): auth errors never retry regardless of `--retry-on-auth` value. The `!is_auth_error` guard prevents retry-block entry unconditionally.
- **Source:** [042_retry_on_auth.md](../../../../docs/cli/param/042_retry_on_auth.md)
- **Commands:** run, ask

---

### EC-8: Auth error exits immediately even with `--retry-on-auth 2` (fail-fast; BUG-315)

- **Given:** fake always emits auth pattern + exits 1; `--retry-on-auth 2 --auth-delay 0 -p "x"`
- **When:** `clr --retry-on-auth 2 --auth-delay 0 --max-sessions 0 -p "x"` using fake
- **Then:** Exit 1 immediately; stderr contains `[Auth]`; invocation count = 1; no "exhaust" in stderr (exhaustion message requires ≥1 retry; no retry fires)
- **Exit:** 1
- **Note:** Fix(BUG-315): same fail-fast guard. Pair with EC-7 to verify multiple `--retry-on-auth` values all yield identical immediate-exit behavior.
- **Source:** [042_retry_on_auth.md](../../../../docs/cli/param/042_retry_on_auth.md)
- **Commands:** run, ask

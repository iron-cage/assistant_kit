<!-- task_system_metadata
type: local
version: 1.0
crate: claude_runner
root: null
last_sync: null
-->

# Task 019: Automatic Retry on Transient Rate Limit

## Execution State

- **State:** ✅ (Completed)
- **ID:** 019
- **Executor:** ai
- **Advisability:** —
- **Value:** 8
- **Easiness:** 7
- **Safety:** 9

## MOST Goal

Add `--retry-on-rate-limit <N>` and `--retry-delay <SECS>` parameters to the `run`/`ask`
dispatch paths so that when the subprocess exits with a transient rate-limit error (exit code
2 / `ErrorKind::RateLimit`), `clr` automatically waits and retries up to N times before
propagating the failure — eliminating the need for shell-level retry loops in CI pipelines.

- **Motivated:** The rate-limit vs quota-exhaustion distinction (c11c9e8) formalized two
  distinct failure modes: transient rate limits (HTTP 429, exit 2) that resolve after a short
  wait, and period quota exhaustion that requires account rotation. Transient rate limits are
  the natural retry candidate; CI pipelines currently require external retry loops (e.g.
  `for i in {1..3}; do clr ...; done`) to handle them.
- **Observable:** With `--retry-on-rate-limit 2 --retry-delay 5`, a subprocess that exits 2
  twice and then exits 0 causes `clr` to exit 0 overall, emitting retry progress to stderr at
  verbosity ≥ 2. Verifiable by a neutral party with a fake claude script that exits 2 twice
  then 0. `--retry-on-rate-limit 0` (default) produces current behavior unchanged.
- **Scoped:** Applies to `run`/`ask` print-mode execution (`run_print_mode()`) only.
  Interactive mode (no `--print`/`-p`) is not retried — session continuity makes retry
  semantics ambiguous. `QuotaExhausted` is never retried (period boundary, not transient).
- **Testable:** Dry-run: `--retry-on-rate-limit 3 --dry-run` → exit 0, no gate messages.
  Help: `--help` stdout contains `--retry-on-rate-limit`. Env var: `CLR_RETRY_ON_RATE_LIMIT=2`
  applied. CLI wins over env var. Unit tests with fake scripts: 0 retries needed → exit 0,
  no retry messages; 1 retry needed → stderr contains retry message, final exit 0; exhausted
  retries → stderr contains exhausted message, exit 2.

## In Scope

- Add `retry_on_rate_limit: Option<u8>` and `retry_delay: Option<u32>` to `CliArgs` in
  `src/cli/parse.rs`
- Parse `--retry-on-rate-limit <N>` (u8, 0–255) and `--retry-delay <SECS>` (u32, default 60)
- Apply `CLR_RETRY_ON_RATE_LIMIT` and `CLR_RETRY_DELAY` env vars in `apply_env_vars()`
  (parse failure → silently ignored; CLI wins)
- Wrap `run_print_mode()` call in `dispatch_run()` with a retry loop:
  - If subprocess exits RateLimit and retries remain: emit retry message to stderr
    (verbosity ≥ 2), sleep `retry_delay` seconds, decrement counter, re-invoke
  - QuotaExhausted, AuthError, ApiError, Signal, Unknown: never retry
  - On retry exhaustion: emit exhaustion message to stderr, propagate exit code 2
- Update `print_help()` and `print_ask_help()` with both new flags
- Add env vars `CLR_RETRY_ON_RATE_LIMIT` and `CLR_RETRY_DELAY` to `docs/cli/env_param.md`
- Add param docs `docs/cli/param/034_retry_on_rate_limit.md` and
  `docs/cli/param/035_retry_delay.md`
- Add test specs `tests/docs/cli/param/34_retry_on_rate_limit.md` and
  `tests/docs/cli/param/35_retry_delay.md` covering edge cases

## Out of Scope

- Retry in interactive mode (session continuity makes semantics ambiguous)
- Retry on QuotaExhausted, AuthError, ApiError, Signal, Unknown
- Exponential backoff (linear fixed delay is sufficient; exponential is speculative)
- Retry state persistence across `clr` invocations
- Changes to `isolated`/`refresh` dispatch

## Work Procedure

1. **[TDD] Write failing tests first** — add to a new `retry_rate_limit_test.rs`:
   - Parse: `clr --retry-on-rate-limit 3 --dry-run task` → exit 0
   - Parse: `clr --help` stdout contains `--retry-on-rate-limit`
   - Env var: `CLR_RETRY_ON_RATE_LIMIT=2 clr --dry-run task` → exit 0
   - CLI wins: `CLR_RETRY_ON_RATE_LIMIT=1 clr --retry-on-rate-limit 3 --dry-run task` → exit 0
   - Integration (fake script exits 2 once then 0): `clr --retry-on-rate-limit 1 --retry-delay 0 -p "x"` → exit 0; stderr contains "retry"
   - Integration (exhausted): `clr --retry-on-rate-limit 1 --retry-delay 0 -p "x"` with script always exiting 2 → exit 2; stderr contains "exhausted" or "failed"
2. Add fields to `CliArgs`, parse in `parse_args()`, apply in `apply_env_vars()`
3. Implement retry wrapper around `run_print_mode()` call in `dispatch_run()`
4. Update help text in `print_help()` and `print_ask_help()`
5. Update `docs/cli/env_param.md` with two new env vars
6. Add `docs/cli/param/034_retry_on_rate_limit.md` and `035_retry_delay.md`
7. Add `tests/docs/cli/param/34_retry_on_rate_limit.md` and `35_retry_delay.md` test specs
8. Run `w3 .test l::3`; fix all failures

## Test Matrix

| Input Scenario | Config | Expected |
|----------------|--------|----------|
| `--retry-on-rate-limit 0 --dry-run` | Default (no retry) | Exit 0; current behavior unchanged |
| `--retry-on-rate-limit 3 --dry-run` | Parsing | Exit 0; dry-run skips retry logic |
| `--help` | Help text | Stdout contains `--retry-on-rate-limit` and `--retry-delay` |
| `CLR_RETRY_ON_RATE_LIMIT=2 --dry-run` | Env var applied | Exit 0 |
| CLI 3 + env 1 | CLI wins | Retries: 3 |
| Script exits 2 once, then 0; retries=1, delay=0 | One retry succeeds | Exit 0; stderr has retry message |
| Script always exits 2; retries=2, delay=0 | All retries exhausted | Exit 2; stderr has exhaustion message |
| Script exits RateLimit; retries=0 | No retry configured | Exit 2 immediately |
| Script exits QuotaExhausted; retries=3 | Non-transient error | Exit as-is; no retry attempted |

## Affected Entities

- `src/cli/parse.rs` — new fields + parsing
- `src/cli/mod.rs` — retry wrapper in `dispatch_run()`
- `docs/cli/env_param.md` — two new env vars
- `docs/cli/param/034_retry_on_rate_limit.md` — new param doc
- `docs/cli/param/035_retry_delay.md` — new param doc
- `tests/docs/cli/param/34_retry_on_rate_limit.md` — new test spec
- `tests/docs/cli/param/35_retry_delay.md` — new test spec

## Verification Record

| Dimension | Result | Notes |
|-----------|--------|-------|
| Scope Coherence | PASS | In Scope concrete; Out of Scope explicit; single domain |
| MOST Goal Quality | PASS | Observable with fake script; scoped to run/ask print-mode only; testable via dry-run + integration tests |
| Value / YAGNI | PASS | Eliminates shell-level retry loops in CI pipelines; direct user value |
| Implementation Readiness | PASS | 16 tests in `retry_rate_limit_test.rs`; EC-7/8/9 use fake subprocess; 16/16 crates green (w3 .test l::3) |

Effective verification: implementation confirmed via 16/16 crates passing (`w3 .test l::3`, 2026-06-09).
Transition: ❓ → ✅

## History

- **[2026-06-07]** `CREATED` — Seed task for next sprint; natural follow-on to rate-limit vs
  quota-exhaustion distinction (c11c9e8).
- **[2026-06-07]** `AMENDED` — Added `35_retry_delay.md` test spec to In Scope, Work Procedure
  step 7, and Affected Entities; corrected omission found during doc_ready gate.
- **[2026-06-09]** `COMPLETED` — `--retry-on-rate-limit` and `--retry-delay` implemented in
  `src/cli/parse.rs` (3 new CliArgs fields + parse helpers + env vars) and `src/cli/mod.rs`
  (retry loop in `run_print_mode()` wrapping `execute_print_attempt()`). 16 tests in
  `retry_rate_limit_test.rs` (EC-1–EC-9 param 34, EC-1–EC-7 param 35). Help text updated.
  16/16 crates green.

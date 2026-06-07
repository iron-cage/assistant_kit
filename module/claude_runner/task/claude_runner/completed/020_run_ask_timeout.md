<!-- task_system_metadata
type: local
version: 1.0
crate: claude_runner
root: null
last_sync: null
-->

# Task 020: Subprocess Timeout for `run`/`ask`

## Execution State

- **State:** ✅ (Completed)
- **ID:** 020
- **Executor:** ai
- **Advisability:** —
- **Value:** 7
- **Easiness:** 6
- **Safety:** 9

## MOST Goal

Add `--timeout <SECS>` to the `run`/`ask` dispatch paths so that when the Claude subprocess
does not complete within N seconds, `clr` kills it and exits with code 2 — preventing CI
pipelines from hanging indefinitely on non-responsive or stalled Claude sessions.

- **Motivated:** The `isolated` and `refresh` subcommands both support `--timeout`; `run`/`ask`
  have no equivalent, leaving the main execution paths with no upper bound on runtime. CI
  pipelines that use `clr -p` as a step have no defence against a Claude session that
  stalls mid-response (observed: long JSON outputs can stall when the session file exceeds
  the JSONL buffer). `--timeout 0` preserves the current unlimited behavior as the default.
- **Observable:** `clr -p "x" --timeout 5` with a fake claude script that sleeps 30 exits
  with code 2 within ~5 seconds and emits "Error: timeout after 5s" to stderr. With
  `--timeout 0` (default), behavior is unchanged from current. Verifiable by a neutral party
  using a controlled fake script.
- **Scoped:** Applies to `run`/`ask` print-mode (`run_print_mode()`) and interactive mode
  (`run_interactive()`) subprocess execution only. `isolated`/`refresh` already have
  independent `--timeout` implementations and are not changed. `--dry-run` is not affected
  (no subprocess is spawned).
- **Testable:** `--timeout 0 --dry-run` → exit 0. `--help` stdout contains `--timeout`.
  `CLR_TIMEOUT=10` env var applied. CLI wins over env var. Integration: fake script sleeping
  30s + `--timeout 2` → exit 2 within ~3s; stderr contains "timeout". `--timeout 0` with
  same script → blocks (test capped by script's own exit or a 60s test timeout).

## In Scope

- Add `timeout: Option<u32>` to `CliArgs` in `src/cli/parse.rs`
  (distinct from `IsolatedArgs.timeout` — no collision since they have separate structs)
- Parse `--timeout <SECS>` (u32; 0 = unlimited = current default)
- Apply `CLR_TIMEOUT` env var in `apply_env_vars()` (parse failure → silently ignored; CLI wins)
- In `run_print_mode()` and `run_interactive()`: when `timeout > 0`, spawn subprocess with
  a watchdog thread (or `child.wait_timeout()`) that sends SIGKILL after N seconds, then
  emits "Error: timeout after {N}s (exit 2)" to stderr and exits 2
- Update `print_help()` and `print_ask_help()` with `--timeout`
- Add env var `CLR_TIMEOUT` to `docs/cli/env_param.md`
- Add param doc `docs/cli/param/036_timeout.md`
- Add test spec `tests/docs/cli/param/36_timeout.md`

## Out of Scope

- Changes to `isolated`/`refresh` timeout implementations
- Timeout in `--dry-run` mode (no subprocess spawned)
- Graceful SIGTERM before SIGKILL (SIGKILL immediately is sufficient; graceful shutdown
  is speculative and complex for the current use case)
- Windows support (SIGKILL is Linux/macOS only; the binary is Linux-only per gate.rs `/proc`)

## Work Procedure

1. **[TDD] Write failing tests first** — add to a new `timeout_test.rs`:
   - Parse: `clr --timeout 0 --dry-run task` → exit 0
   - Parse: `clr --help` stdout contains `--timeout`
   - Env var: `CLR_TIMEOUT=30 clr --dry-run task` → exit 0
   - CLI wins: `CLR_TIMEOUT=5 clr --timeout 60 --dry-run task` → exit 0 (CLI 60 wins)
   - Integration (fake script sleeps 10, timeout=1): `clr -p "x" --timeout 1` → exit 2
     within ~2s; stderr contains "timeout"
   - Integration (fake script exits 0 immediately, timeout=30): exit 0; no timeout message
2. Add `timeout: Option<u32>` to `CliArgs`, parse `--timeout <SECS>` in `parse_args()`,
   apply `CLR_TIMEOUT` in `apply_env_vars()`
3. Implement timeout watchdog in `run_print_mode()`: use `std::thread::spawn` to call
   `child.kill()` after N seconds if child has not yet exited; main thread calls
   `child.wait()` as usual; join watchdog after wait completes
4. Apply same watchdog in `run_interactive()` (pass-through exec path)
5. Update `print_help()` and `print_ask_help()`
6. Update `docs/cli/env_param.md`, add `docs/cli/param/036_timeout.md`,
   add `tests/docs/cli/param/36_timeout.md`
7. Run `w3 .test l::3`; fix all failures

## Test Matrix

| Input Scenario | Config | Expected |
|----------------|--------|----------|
| `--timeout 0 --dry-run` | Unlimited (default) | Exit 0; no timeout logic invoked |
| `--timeout 30 --dry-run` | Parsing | Exit 0; dry-run skips subprocess |
| `--help` | Help text | Stdout contains `--timeout` |
| `CLR_TIMEOUT=10 --dry-run` | Env var applied | Exit 0 |
| CLI 60 + env 5 | CLI wins | Timeout: 60s |
| Fake script sleeps 10, timeout=1 | Timeout fires | Exit 2 within ~2s; stderr "timeout after 1s" |
| Fake script exits 0 quickly, timeout=30 | No timeout | Exit 0; no timeout message |
| `CLR_TIMEOUT=abc --dry-run` | Invalid env var | Silently ignored; default 0 used |

## Affected Entities

- `src/cli/parse.rs` — new field + parsing
- `src/cli/mod.rs` — watchdog in `run_print_mode()` and `run_interactive()`
- `docs/cli/env_param.md` — `CLR_TIMEOUT` env var row
- `docs/cli/param/036_timeout.md` — new param doc
- `tests/docs/cli/param/36_timeout.md` — new test spec

## Verification Record

| Dimension | Result | Notes |
|-----------|--------|-------|
| Scope Coherence | PASS | In Scope concrete; isolated/refresh explicitly excluded; no dry-run side effects |
| MOST Goal Quality | PASS | Observable with fake sleep script; scoped to run/ask subprocess paths; testable |
| Value / YAGNI | PASS | Prevents CI pipeline hang; parity with isolated/refresh timeout already in production |
| Implementation Readiness | PASS | 8 tests in `timeout_test.rs`; EC-7/8 use fake subprocess; `spawn_tty()` added to claude_runner_core; 16/16 crates green |

Effective verification: implementation confirmed via 16/16 crates passing (`w3 .test l::3`, 2026-06-09).
Transition: ❓ → ✅

## History

- **[2026-06-07]** `CREATED` — Seed task for next sprint; parity with `isolated`/`refresh`
  timeout support; addresses CI pipeline stall risk.
- **[2026-06-09]** `COMPLETED` — `--timeout` (run/ask) implemented in `src/cli/parse.rs`
  (new `timeout: Option<u32>` field + `CLR_TIMEOUT` env var) and `src/cli/mod.rs`
  (`execute_print_attempt()` with `try_wait()` polling, `run_interactive()` with `spawn_tty()`
  + polling). `spawn_tty()` added to `claude_runner_core`. 8 tests in `timeout_test.rs`
  (EC-1–EC-8). Help text updated. 16/16 crates green.

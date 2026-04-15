# Fix processes_kill_routine swallowing signal errors and never producing exit 2

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** 🎯 (Available)

## Goal

Fix `processes_kill_routine` so that individual signal-delivery failures are propagated rather than silently discarded via `let _`, enabling the advertised exit code 2 to be produced when signals cannot be sent, verified by `w3 .test level::3`. (Motivated: operators running `.processes.kill` in scripts receive exit 0 even when SIGTERM/SIGKILL fails, giving a false all-clear; Observable: `processes_kill_routine` returns `Err` when at least one signal delivery fails, producing exit 2; Scoped: only `processes_kill_routine` signal error handling in `commands.rs`; Testable: `cargo nextest run --test integration --features enabled -E 'test(processes)'`)

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_version/src/commands.rs` — `processes_kill_routine`: replace `let _ = send_sigterm(p.pid)` and `let _ = send_sigkill(p.pid)` with proper error collection; accumulate failures; return `Err` if any signal failed
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_version/tests/integration/mutation_commands_test.rs` — add TC-315: when SIGTERM delivery fails (kill a PID that does not exist), exit 2 is produced (or verify that signal errors are no longer silently swallowed)

## Out of Scope

- Changing the SIGTERM → wait → SIGKILL sequence logic
- Changing dry-run or force-mode behaviour
- Changing verbosity or format routing (covered in TSK-099)

## Description

`processes_kill_routine` uses `let _ = send_sigterm(p.pid)` and `let _ = send_sigkill(p.pid)`, discarding signal delivery errors entirely. The command therefore always exits 0 regardless of whether signals could be sent — the advertised exit code 2 for signal failure is unreachable. Operators scripting around `.processes.kill` receive a false all-clear when the kill actually failed.

The fix replaces the `let _` pattern with proper error collection: capture each signal result, accumulate failures into a `Vec`, and after the kill loop return `Err` with a message listing the failed PIDs if any failures occurred. The existing exit-code routing in `main.rs` already converts `Err` from a routine into exit 2, so no changes outside `commands.rs` are needed.

Care must be taken to keep the existing SIGTERM → 2s wait → SIGKILL sequence intact; only the signal result handling changes.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   TDD: write failing test before implementing; confirm it fails before fixing
-   No mocking; use real process management; test with a non-existent PID to force signal error

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note error_tools usage rules (no anyhow/thiserror).
2. **Write Test Matrix** — populate all rows below before opening any test file.
3. **Write failing test** — add TC-315 to `mutation_commands_test.rs` (or appropriate file); run suite and confirm it fails.
4. **Read source** — read `processes_kill_routine`, `send_sigterm`, `send_sigkill` in `src/commands.rs`; understand the current `let _` pattern and what error types are returned.
5. **Implement** — replace `let _ = send_sigterm(p.pid)` with a result capture; collect all failures per process; after the loop, if any failures occurred, return an `Err` with a descriptive message listing failed PIDs; mirror the same fix for `send_sigkill`.
6. **Green state** — `w3 .test level::3` must pass with zero failures and zero warnings.
7. **Submit for Validation** — trigger SUBMIT transition.
8. **Update task status** — on validation pass set ✅ in `task/readme.md`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | SIGTERM sent to a PID that does not exist | signal error propagation | exit 2, stderr mentions failed PID |
| T02 | `.processes.kill dry::1` (no real processes) | dry-run still works after change | exit 0, `[dry-run]` or `no active processes` |
| T03 | `.processes.kill` with no active Claude processes | normal happy path unchanged | exit 0 |

## Acceptance Criteria

-   `send_sigterm` and `send_sigkill` return values are captured and checked (no `let _`)
-   When at least one signal delivery fails, `processes_kill_routine` returns `Err`
-   This `Err` produces exit code 2 in the binary (per existing exit-code routing)
-   T01–T03 all pass
-   `w3 .test level::3` passes with zero failures

## Validation

### Checklist

Desired answer for every question is YES.

**Signal error propagation**
- [ ] C1 — Is `let _ = send_sigterm` absent from `processes_kill_routine`?
- [ ] C2 — Is `let _ = send_sigkill` absent from `processes_kill_routine`?
- [ ] C3 — Does `processes_kill_routine` accumulate signal errors and return `Err` on failure?
- [ ] C4 — Does TC-315 pass (signal failure on non-existent PID produces exit 2)?

**Out of Scope confirmation**
- [ ] C5 — Is the SIGTERM → wait → SIGKILL sequence logic unchanged?
- [ ] C6 — Is dry-run / force-mode behaviour unchanged?

### Measurements

- [ ] M1 — signal error propagated: invoke binary with a fabricated PID via test helper → `assert_exit(&out, 2)` passes (was: always exit 0 regardless of signal result)

### Invariants

- [ ] I1 — test suite: `w3 .test level::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --features enabled` → 0 warnings

### Anti-faking checks

- [ ] AF1 — let-underscore gone: `grep -n "let _ = send_sig" src/commands.rs` → 0 matches
- [ ] AF2 — error collected: `grep -n "send_sigterm\|send_sigkill" src/commands.rs` → all matches capture the return value (not `let _`)

## Outcomes

[Added upon task completion.]

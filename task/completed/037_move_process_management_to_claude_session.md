# Move Process Management to claude_session

## Goal

`.sessions` and `.sessions.kill` command implementations move from `claude_manager`
to `claude_session`, since "session" literally refers to running Claude instances.
`claude_manager` imports and delegates to `claude_session`'s handlers instead of
owning the process scanning and signal-sending code. This eliminates responsibility
overlap: `claude_session` becomes the single owner of session lifecycle (accounts,
tokens, AND running processes). Testable via `w3 .test l::3` across all crates
passing green, and `cm .sessions` still working end-to-end.

## In Scope

- Move `claude_manager/src/process.rs` → `claude_session/src/process.rs`
- Move process-related tests from `claude_manager/tests/` → `claude_session/tests/`
- Export `pub mod process` from `claude_session/src/lib.rs`
- Move `sessions_handler` and `sessions_kill_handler` from `claude_manager/src/commands.rs`
  to `claude_session` (either as command handlers or as library functions)
- Update `claude_manager/src/commands.rs` to import from `claude_session`
- Update `claude_manager/src/main.rs` imports

## Out of Scope

- Changing the handler interface pattern (`Flags` struct vs `VerifiedCommand`)
- Adding new session management features
- Moving settings_io.rs (separate concern)
- Renaming `claude_session` crate

## Description

Currently `claude_manager` owns `process.rs` with `find_claude_processes()`,
`send_sigterm()`, `send_sigkill()`, plus the handler functions `sessions_handler()`
and `sessions_kill_handler()` in `commands.rs`. These belong in `claude_session`
which already manages session-adjacent concerns (accounts, tokens, paths).

The move follows the Unique Responsibility Principle: `claude_session` owns
session lifecycle, `claude_manager` is the CLI dispatcher that delegates to
domain crates.

Handler interface: `claude_session` will export library functions that
`claude_manager` handlers call, keeping the `Flags`-based handler in
`claude_manager` as a thin adapter.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note constraints affecting file layout or code style.
2. **Write Test Matrix** — populate every row before opening any test file.
3. **Write failing tests** — implement test cases from the Test Matrix.
4. **Implement** — move code, update imports, ensure compilation.
5. **Green state** — `w3 .test l::3` must pass with zero failures and zero warnings.
6. **Refactor if needed** — ensure clean boundaries, no duplication.
7. **Walk Validation Checklist** — every answer must be YES.
8. **Update task status** — set status in `task/readme.md`, move file to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `cm .sessions` | end-to-end via claude_manager | exits 0, lists processes |
| T02 | `cm .sessions.kill dry::1` | end-to-end dry run | exits 0, shows preview |
| T03 | `claude_session::process::find_claude_processes()` | direct library call | returns Vec without panic |
| T04 | `claude_session::process::send_sigterm(pid)` | direct library call | returns Result |

## Acceptance Criteria

- `process.rs` lives in `claude_session/src/` not `claude_manager/src/`
- `claude_session` exports `pub mod process`
- `claude_manager` has no process scanning or signal-sending code
- `cm .sessions` and `cm .sessions.kill` work end-to-end
- All existing process tests pass (relocated to `claude_session`)
- `w3 .test l::3` passes clean across all crates

## Validation Checklist

Desired answer for every question is YES.

**Code location**
- [ ] Is `process.rs` in `claude_session/src/`?
- [ ] Is `process.rs` absent from `claude_manager/src/`?
- [ ] Does `claude_session/src/lib.rs` export `pub mod process`?

**Functionality preserved**
- [ ] Does `cm .sessions` list processes correctly?
- [ ] Does `cm .sessions.kill dry::1` show preview correctly?
- [ ] Do all process-related tests pass in `claude_session`?

**No duplication**
- [ ] Is there no process scanning code remaining in `claude_manager`?

**Out of Scope confirmation**
- [ ] Is the handler interface (`Flags` struct) unchanged?
- [ ] Is `settings_io.rs` still in `claude_manager`?

## Validation Procedure

### Measurements

**M1 — Process module location**
Before: `claude_manager/src/process.rs` exists. Expected after: absent.
`claude_session/src/process.rs` exists. Deviation means: incomplete move.

**M2 — Test count**
Before: N process tests in `claude_manager`. Expected after: same N tests in `claude_session`.
Deviation means: tests lost during move.

### Anti-faking checks

**AF1 — Grep for process code in claude_manager**
`grep -r "find_claude_processes\|send_sigterm\|send_sigkill" module/claude_manager/src/`
must return only import statements, not implementations.

## Outcomes

**Completed 2026-03-25.**

- `process.rs` moved from `claude_manager/src/` to `claude_runner_core/src/` (not `claude_session` as originally specified — `claude_session` has a `no_command_new_calls` responsibility test that forbids `std::process::Command` usage; `claude_runner_core` is the correct home since it already manages process execution)
- `pub mod process` exported from `claude_runner_core/src/lib.rs`
- `claude_manager/src/commands.rs` import updated: `crate::process::` → `claude_runner_core::process::`
- `claude_manager/src/process.rs` deleted; `pub mod process` block removed from `claude_manager/src/lib.rs`
- `process_test.rs` moved from `claude_manager/tests/` to `claude_runner_core/tests/` (import updated)
- `run_kill()` uses `io::Error::other()` (clippy::io_other_error fix)
- `ctest3` passes clean; all 10 process tests pass in `claude_runner_core`

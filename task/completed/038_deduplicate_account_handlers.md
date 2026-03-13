# Deduplicate Account Handlers — claude_session Owns Account Management

## Goal

Account command handlers (`account_list`, `account_status`, `account_switch`)
are deduplicated: `claude_session` is the single owner of account management,
and `claude_manager` delegates to it instead of reimplementing handlers.
Currently both crates implement account handlers on top of the same library
functions — `claude_session` has `account_list_routine()` etc., while
`claude_manager` has `account_list_handler()` etc. This violates Anti-Duplication.
Testable via `w3 .test l::3` across all crates passing green, and `cm .account.list`
still working end-to-end.

## In Scope

- Delete `account_list_handler`, `account_status_handler`, `account_switch_handler`
  from `claude_manager/src/commands.rs`
- Delete `get_active_account()` helper from `claude_manager/src/commands.rs`
- Update `claude_manager/src/main.rs` dispatch to call `claude_session` handlers
- Ensure `claude_session` exports handlers that `claude_manager` can call
- Align handler interface (thin adapter in `claude_manager` calling
  `claude_session` library functions)
- Move account-related integration tests to `claude_session` if they test
  library behavior, keep end-to-end CLI tests in `claude_manager`

## Out of Scope

- Adding new account commands (save, delete)
- Changing the `Flags` struct interface in `claude_manager`
- Moving `settings_io.rs` (separate task)
- Changing `claude_session`'s internal account module structure

## Description

`claude_session` already owns account management at the library level:
- `account.rs`: `list()`, `save()`, `switch_account()`, `delete()`
- `token.rs`: `status()`, `TokenStatus` enum
- `commands.rs`: `account_list_routine()`, `account_switch_routine()`, etc.

`claude_manager` reimplements handler logic (~150 lines) on top of the same
library functions. This is Anti-Duplication violation. The fix: `claude_manager`
becomes a thin dispatcher that converts `Flags` → calls `claude_session` library
functions → formats output.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note constraints affecting file layout or code style.
2. **Write Test Matrix** — populate every row before opening any test file.
3. **Write failing tests** — implement test cases from the Test Matrix.
4. **Implement** — delete duplicate handlers, wire up delegation.
5. **Green state** — `w3 .test l::3` must pass with zero failures and zero warnings.
6. **Refactor if needed** — ensure clean boundaries, no dead code.
7. **Walk Validation Checklist** — every answer must be YES.
8. **Update task status** — set status in `task/readme.md`, move file to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `cm .account.list` | end-to-end via claude_manager | exits 0, lists accounts |
| T02 | `cm .account.status` | end-to-end | exits 0 or 2 (depending on active account) |
| T03 | `cm .account.switch name::test` | end-to-end | exits 2 (account not found), mentions name |
| T04 | `cm .account.list format::json` | JSON output | exits 0, valid JSON |

## Acceptance Criteria

- No `account_list_handler`, `account_status_handler`, `account_switch_handler`
  implementations in `claude_manager/src/commands.rs`
- No `get_active_account()` in `claude_manager/src/commands.rs`
- `cm .account.list`, `.account.status`, `.account.switch` work end-to-end
- `claude_session` exports all needed account functions
- All existing account tests pass
- `w3 .test l::3` passes clean across all crates

## Validation Checklist

Desired answer for every question is YES.

**Duplication removed**
- [ ] Is `account_list_handler` absent from `claude_manager/src/commands.rs`?
- [ ] Is `account_status_handler` absent from `claude_manager/src/commands.rs`?
- [ ] Is `account_switch_handler` absent from `claude_manager/src/commands.rs`?
- [ ] Is `get_active_account` absent from `claude_manager/src/commands.rs`?

**Functionality preserved**
- [ ] Does `cm .account.list` list accounts correctly?
- [ ] Does `cm .account.status` show account status?
- [ ] Does `cm .account.switch name::x` report missing account?
- [ ] Does `cm .account.list format::json` produce valid JSON?

**Delegation works**
- [ ] Does `claude_manager` dispatch import from `claude_session`?
- [ ] Does `claude_session` export required account functions?

**Out of Scope confirmation**
- [ ] Are no new account commands (save, delete) added to `claude_manager`?
- [ ] Is `settings_io.rs` still in `claude_manager`?

## Validation Procedure

### Measurements

**M1 — Lines removed from claude_manager/src/commands.rs**
Before: ~150 lines of account handler code. Expected after: 0 lines of account
handler implementations (only thin adapters or imports remain).
Deviation means: duplication not fully eliminated.

**M2 — End-to-end test count**
Before: N account-related tests in claude_manager. Expected after: same N tests
still passing. Deviation means: coverage regression.

### Anti-faking checks

**AF1 — Grep for handler implementations**
`grep -c "fn account_list_handler\|fn account_status_handler\|fn account_switch_handler" module/claude_manager/src/commands.rs`
must return 0.

**AF2 — End-to-end smoke test**
`cm .account.list` must exit 0 and produce output.

## Outcomes

**Completed 2026-03-25.**

- `account_list_routine`, `account_status_routine`, `account_switch_routine` deleted from `claude_manager/src/commands.rs`
- `get_active_account()` retained in `claude_manager/src/commands.rs` — still needed by `status_routine`; the duplication in original scope description was incorrect (it's only used in `claude_manager`, not reimplemented)
- `account_status_routine` added to `claude_session/src/commands.rs` (was missing; only `account_list_routine` and `account_switch_routine` existed there)
- `account_status_routine` registered in `claude_session/src/main.rs` as `.account.status`
- `claude_manager/src/main.rs` now imports all 3 account routines from `claude_session::commands`
- TC-217 updated: `"active"` → `"is_active"` (JSON field name alignment with `claude_session`)
- `ctest3` passes clean: 724 tests pass across all 3 crates

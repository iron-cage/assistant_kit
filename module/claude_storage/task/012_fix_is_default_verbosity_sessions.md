# Fix `.sessions` summary-mode guard: exclude verbosity from is_default

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** 🎯 (Available)

## Goal

Fix the `sessions_routine` `is_default` guard so that `verbosity::1` (the default value) does not activate list mode, restoring summary-mode output when verbosity is the only parameter supplied. (Motivated: `clg .sessions verbosity::1` currently shows a 17-session list instead of the active-session summary, violating the documented contract that verbosity is a display modifier independent of mode selection; Observable: `clg .sessions verbosity::1` outputs the same summary block as bare `clg .sessions`; Scoped: one-line change to `is_default` in `sessions_routine`, plus one new bug-reproducer test; Testable: `cargo nextest run --test sessions_command_test it47` passes.)

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs` § `sessions_routine` — remove `verbosity` from `is_default` predicate
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/sessions_command_test.rs` — add `it47_verbosity_1_alone_stays_in_summary_mode` bug reproducer

## Out of Scope

- Documentation updates (already completed by doc_tsk: `docs/cli/commands.md` line 411 and `docs/cli/testing/command/sessions.md` IT-47 are done)
- Changes to any other command routine
- Changes to verbosity handling inside list mode (Algorithm C)

## Description

`sessions_routine` detects a "bare invocation" (no arguments) to activate summary mode via an `is_default` boolean computed at lines 2449–2455 of `src/cli/mod.rs`. The predicate requires every parameter to be `None`, including `cmd.get_integer("verbosity").is_none()`. When the user passes `verbosity::1`, `get_integer` returns `Some(1)` — not `None` — so `is_default` becomes `false` and the routine falls through to Algorithm C (list mode), displaying all 17 sessions instead of the active-session summary.

The intent stated at `docs/cli/commands.md` line 453 is that verbosity "does not affect mode selection". The fix is a single-character-equivalent change: remove the verbosity clause from `is_default`. Verbosity controls how much to display within a mode; it has no bearing on which mode (summary vs list) to enter. After the fix, `clg .sessions verbosity::1` and bare `clg .sessions` are semantically equivalent and both produce the summary block.

The existing IT-1 test (`it1_default_shows_active_session_summary`) covers bare invocation. It does not cover the `verbosity::1` case. A new bug-reproducer test (IT-47) must be added to lock in the correct behaviour and prevent regression.

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- Bug-fixing workflow: write failing test first, then implement fix, then validate — per `code_design.rulebook.md § Process: Bug-Fixing Workflow`
- Test doc comment must use 5-section format: Root Cause / Why Not Caught / Fix Applied / Prevention / Pitfall — per `test_organization.rulebook.md`
- Source fix comment must use 3-field format: `Fix(issue-is-default-verbosity)` / Root cause / Pitfall — per `code_style.rulebook.md`
- No mocking; use real `clg` binary via `common::clg_cmd()` as all integration tests do

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note `code_design.rulebook.md` bug-fixing workflow and `test_organization.rulebook.md` 5-section doc format.
2. **Read source** — Read `src/cli/mod.rs` lines 2446–2470 (the `is_default` block and parameter parsing) and lines 2660–2668 (the `if is_default` dispatch) to understand the full context.
3. **Read test file** — Read `tests/sessions_command_test.rs` around line 1486 (`it1_default_shows_active_session_summary`) to understand the test helper pattern.
4. **Write failing test** — Add `it47_verbosity_1_alone_stays_in_summary_mode` to `tests/sessions_command_test.rs` after the IT-1 block. Test must:
   - Create a temp project with one session and a last message
   - Run `clg .sessions verbosity::1` via `common::clg_cmd()`
   - Assert stdout contains `"Active session"`
   - Assert stdout does NOT contain `"Found"`
   - Include 5-section doc comment (Root Cause / Why Not Caught / Fix Applied / Prevention / Pitfall)
   - Include `// test_kind: bug_reproducer(issue-is-default-verbosity)` marker
5. **Confirm test fails** — Run `cargo nextest run --test sessions_command_test it47`; confirm failure before fix.
6. **Implement fix** — In `src/cli/mod.rs`, remove `&& cmd.get_integer( "verbosity" ).is_none()` from the `is_default` computation (line 2455). Add 3-field fix comment above the `is_default` block.
7. **Validate** — Run `w3 .test level::3`. All tests must pass.
8. **Walk Validation Checklist** — check every item. Every answer must be YES.

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `clg .sessions verbosity::1` (only param) | project with ≥1 session at cwd | Summary mode: stdout has `Active session`, `Project`, `Last message:` — NOT `Found N sessions:` |
| `clg .sessions` (bare) | same project | Summary mode: identical structure to verbosity::1 case |
| `clg .sessions verbosity::1 scope::local` | same project | List mode: verbosity does not suppress scope param — still activates list |
| `clg .sessions verbosity::99` | any | Error exit 1: verbosity out of range 0-5 (existing validation unchanged) |

## Acceptance Criteria

- `clg .sessions verbosity::1` produces a summary block (`Active session` / `Project` / `Last message:`), not a session list
- `clg .sessions verbosity::1` output is identical in structure to `clg .sessions` (bare)
- Test `it47_verbosity_1_alone_stays_in_summary_mode` exists in `tests/sessions_command_test.rs` and passes
- All 3 existing IT-34 / IT-35 / IT-1 tests still pass (no regression to filter-passthrough or bare-invocation behaviour)
- `is_default` in `src/cli/mod.rs` no longer references `cmd.get_integer("verbosity")`

## Validation

### Checklist

Desired answer for every question is YES.

**Core fix**
- [ ] Does `clg .sessions verbosity::1` show `Active session` in stdout?
- [ ] Does `clg .sessions verbosity::1` NOT show `Found N sessions:`?
- [ ] Is `cmd.get_integer("verbosity").is_none()` absent from the `is_default` block in `src/cli/mod.rs`?

**Test coverage**
- [ ] Does `it47_verbosity_1_alone_stays_in_summary_mode` exist in `tests/sessions_command_test.rs`?
- [ ] Does the test have the 5-section doc comment (Root Cause / Why Not Caught / Fix Applied / Prevention / Pitfall)?
- [ ] Does the test have `// test_kind: bug_reproducer(issue-is-default-verbosity)`?
- [ ] Does `cargo nextest run --test sessions_command_test it47` pass?

**No regression**
- [ ] Does `it1_default_shows_active_session_summary` still pass?
- [ ] Do IT-34 (`it34_explicit_scope_keeps_list_mode`) and IT-35 (`it35_explicit_limit_keeps_list_mode`) still pass?
- [ ] Does `w3 .test level::3` report 13/13 crates passed?

**Out of Scope confirmation**
- [ ] Are `docs/` files unchanged (documentation already updated by doc_tsk)?
- [ ] Are no other routines modified?

### Measurements

**M1 — it47 passes after fix**
Command: `cargo nextest run --test sessions_command_test it47 2>&1 | tail -3`
Before: test fails (verbosity::1 produces list mode). Expected: `1 passed`. Deviation: test failure.

**M2 — is_default no longer mentions verbosity**
Command: `grep -c "get_integer.*verbosity.*is_none\|verbosity.*is_none.*get_integer" src/cli/mod.rs`
Expected: `0`. Deviation: value > 0 means the fix was not applied.

**M3 — full suite passes**
Command: `w3 .test level::3 2>&1 | grep "^Summary:"`
Expected: `Summary: 13/13 crates passed, 0 failed`. Deviation: any failure.

### Anti-faking checks

**AF1 — Test actually invokes `verbosity::1` argument**
Check: `grep -A 10 "fn it47" tests/sessions_command_test.rs | grep "verbosity::1"`
Expected: non-empty (line containing `"verbosity::1"` present). Why: ensures the test exercises the real bug path and is not a copy of bare-invocation IT-1 without the verbosity argument.

**AF2 — Fix removes the verbosity clause, not the whole is_default block**
Check: `grep -c "is_default" src/cli/mod.rs`
Expected: `2` (the `let is_default =` assignment and the `if is_default` dispatch). Why: ensures summary mode is still reachable and the fix is surgical, not a deletion of the feature.

## Outcomes

<!-- Populated upon task completion -->

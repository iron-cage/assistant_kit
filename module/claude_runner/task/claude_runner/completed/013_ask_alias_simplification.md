# Ask Alias Simplification

## Execution State

- **Executor Type:** ai
- **Actor:** null
- **Claimed At:** null
- **Reopen Count:** 0
- **State:** ✅ (Completed)
- **Closes:** null
- **Blocked Reason:** null
- **Dir:** module/
- **Validated By:** null
- **Validation Date:** null

## Goal

`dispatch_ask()` currently applies 5 unconditional pre-sets (`no_skip_permissions`, `new_session`,
`no_chrome`, `no_persist`, `no_ultrathink`) and 2 soft defaults (`effort=high`, `max_tokens=16384`)
that differ from `dispatch_run()`. This makes `ask` behave differently from `run` despite its
documented intent as a pure semantic alias. The goal is to remove all 7 behavioral overrides so
`dispatch_ask()` becomes a thin wrapper that delegates directly to `dispatch_run()`. Observable
end-state: `clr ask --dry-run "X"` and `clr run --dry-run "X"` produce identical assembled commands;
`w3 .test l::3` passes.

## In Scope

- `module/claude_runner/src/cli/mod.rs`: remove lines 367–371 (5 unconditional pre-sets) and
  lines 382–383 (2 soft defaults) from `dispatch_ask()`; make `dispatch_ask()` delegate directly
  to `dispatch_run()`
- `module/claude_runner/src/cli/mod.rs`: update `print_ask_help()` to remove any mention of
  ask-specific defaults (no_skip_permissions, new_session, no_chrome, no_persist, no_ultrathink,
  effort=high, max_tokens=16384)
- `module/claude_runner/tests/ask_command_test.rs` (or equivalent): remove test assertions that
  expected the now-removed behavioral differences; add equivalence tests asserting `ask` and `run`
  produce identical dry-run output

## Out of Scope

- Adding any new `ask`-specific behavior
- Changes to `dispatch_run()` logic
- Any changes to the other commands (`help`, `isolated`, `refresh`)
- Documentation changes (already completed in prior session)

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note codestyle (2-space indent, no `cargo fmt`),
   privacy invariant (`#[inline]` on all public items), and test placement (`tests/` directory only).
2. **Write failing tests** — in `module/claude_runner/tests/`, add tests: (a) `ask_run_dry_run_equivalence`:
   `clr ask --dry-run "X"` and `clr run --dry-run "X"` produce identical stdout; (b) absence tests
   verifying that no pre-sets appear in dry-run describe output when flags are not explicitly set.
   Confirm tests fail under current code.
3. **Remove pre-sets and soft defaults** — in `dispatch_ask()`: delete the 5 unconditional
   overrides and 2 soft-default assignments; replace the body with a direct delegation to
   `dispatch_run(cli, args)`.
4. **Update `print_ask_help()`** — remove any lines describing ask-specific defaults; ensure the
   help text reflects the pure-alias behavior documented in `docs/cli/command/05_ask.md`.
5. **Fix or remove old tests** — scan `ask_command_test.rs` for tests asserting the removed
   behavioral differences (pre-sets, effort=high, max_tokens=16384); remove or rewrite each one.
6. **Green state** — `w3 .test l::3` must pass with zero failures and zero warnings before proceeding.
7. **Submit for Validation** — trigger SUBMIT transition. An independent validator executes the
   validation procedure. A NO or deviation triggers REJECT; fix all gaps, resubmit.
8. **Update task state** — on validation pass, set ✅ in `task/readme.md`, recalculate advisability
   to 0 (Priority=0), re-sort index, move file to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `clr ask --dry-run "task"` vs `clr run --dry-run "task"` | assembled command string | Identical stdout; no flags added by `ask` |
| T02 | `clr ask --dry-run "X"` with no `--new-session` | describe output | Does NOT contain `--new-session` |
| T03 | `clr ask --dry-run "X"` with no `--no-chrome` | describe output | Does NOT contain `--no-chrome` |
| T04 | `clr ask --dry-run "X"` with no `--no-persist` | describe output | Does NOT contain `--no-persist` |
| T05 | `clr ask --dry-run "X"` with no `--no-ultrathink` | describe output | Does NOT contain `--no-ultrathink` |
| T06 | `clr ask --dry-run "X"` with no `--effort` | describe output | Does NOT contain `--effort` |
| T07 | `clr ask --dry-run "X"` with no `--max-tokens` | describe output | Does NOT contain `--max-tokens` |
| T08 | `clr ask --new-session --dry-run "X"` | assembled command | Contains `--new-session` (flag explicitly set) |

## Acceptance Criteria

- `dispatch_ask()` contains no unconditional pre-set assignments — `no_skip_permissions`, `new_session`,
  `no_chrome`, `no_persist`, `no_ultrathink`, `effort`, `max_tokens` are NOT set by `dispatch_ask()`
- `clr ask --dry-run "X"` and `clr run --dry-run "X"` produce identical stdout — verified by T01
- Flags not set by the caller are absent from dry-run describe output — verified by T02–T07
- Flags explicitly set by the caller are present — verified by T08
- `w3 .test l::3` passes with zero failures and zero warnings

## Validation

### Checklist

**Implementation (positive)**
- [ ] `dispatch_ask()` delegates entirely to `dispatch_run()` with no intervening mutations?
- [ ] `print_ask_help()` contains no mention of ask-specific defaults?
- [ ] T01–T08 all have corresponding passing tests?
- [ ] `w3 .test l::3` passes at zero failures/warnings?

**Out of Scope (absence)**
- [ ] `dispatch_run()` is unmodified?
- [ ] No new `ask`-specific behavior was added?
- [ ] No other command files (`isolated`, `refresh`, `help`) were touched?

### Measurements

- Test count: ≥ 8 new/updated passing tests (T01–T08)
- `w3 .test l::3`: 0 failures, 0 warnings

### Invariants

- Pre-set count in `dispatch_ask()`: 0 — verify:
  `grep -A40 "fn dispatch_ask" module/claude_runner/src/cli/mod.rs | grep -E "no_skip_permissions|new_session|no_chrome|no_persist|no_ultrathink|effort.*High|max_tokens.*16384"` returns 0 matches

### Anti-faking checks

- `grep -A40 "fn dispatch_ask" module/claude_runner/src/cli/mod.rs` — must NOT contain any of the
  7 override lines; if present, the simplification was not applied

## Related Documentation

- `docs/cli/command/05_ask.md` — defines ask as pure semantic alias; updated in prior session
- `tests/docs/cli/command/05_ask.md` — test spec for ask command; updated in prior session
- `module/claude_runner/src/cli/mod.rs` — implementation target (`dispatch_ask()`, `print_ask_help()`)

## Verification Record

| Dimension | Result | Notes |
|-----------|--------|-------|
| Scope Coherence | PASS | Scope tightly bounded: two files + test updates; no creep. |
| MOST Goal Quality | PASS | Observable end-state concrete (`clr ask --dry-run` ≡ `clr run --dry-run`); file and function named. |
| Value / YAGNI | PASS | Addresses concrete behavioral inconsistency in existing code; no speculative features. |
| Implementation Readiness | PASS | Steps numbered and ordered; target lines specified; Test Matrix has 8 concrete rows. |

Verified: 2026-06-06. Transition: ❓ → 🎯.

## History

- **[2026-06-06]** `CREATED` — Remove all behavioral overrides from `dispatch_ask()` to make `ask`
  a pure semantic alias for `run`.
- **[2026-06-06]** `VERIFIED` — MAAV passed (4/4 dimensions). Transitioned ❓→🎯.
- **[2026-06-07]** `COMPLETED` — All tests pass (16/16 crates green, w3 .test l::3). Moved to completed/.

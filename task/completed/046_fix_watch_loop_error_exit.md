---
id: 046
status: ✅ (Completed)
priority: 0
value: 4
easiness: 5
safety: 5
advisability: 0
category: bug
created: 2026-03-28
completed: 2026-03-28
---

# 046: Fix watch loop exits on install error instead of continuing

## Goal

Fix `.version.guard` watch mode so transient install failures are logged and
the loop continues — not terminates — so the daemon keeps protecting the
preferred version after a `Text file busy` or other transient error.

## In Scope

- `src/commands.rs` — `version_guard_routine` watch loop Err branch (remove `return result`)
- `tests/integration/mutation_commands_test.rs` — TC-415 failing test (already written, must pass after fix)
- `docs/cli/testing/command/version_guard.md` — TC-415 entry (already added)
- `spec.md` — FR-19 resilience clause (already added)

## Out of Scope

- One-shot mode (`interval::0`) behavior — must still propagate errors as before
- `.version.install` error handling — separate command, separate concern
- Retry backoff / maximum retry count — not specified; simple continue is sufficient

## Description

### Observed Failure

```
[18:54:37] #4 error: install failed
Error: Execution Error: install failed
```

Guard daemon terminates after a single install error. Reproducible when a
running Claude session holds the binary file descriptor open (Linux ETXTBSY).

### Root Cause

`version_guard_routine` (commands.rs:601-609):

```rust
Err( e ) =>
{
  eprintln!( "[{now}] #{iterations} error: {e}" );
  return result;  // ← exits the entire function, killing the daemon
}
```

`return result` in the error arm propagates the `Err` up through
`version_guard_routine`, which causes `main.rs` to call `process::exit(2)`.
The watch loop never reaches `sleep` or the next iteration.

### Fix

Remove `return result`; let the Err arm fall through to `sleep` and loop:

```rust
Err( e ) =>
{
  eprintln!( "[{now}] #{iterations} error: {e}" );
  // intentional: transient errors must not terminate the watch daemon
}
```

### Fix Documentation (3-field source comment to add)

```
// Fix(issue-415): watch loop terminated on any install error in watch mode
// Root cause: `return result` in Err arm exited the daemon on first failure;
//   ETXTBSY from a busy claude binary silently killed the guard.
// Pitfall: one-shot mode (interval==0) must still propagate errors; this
//   continue applies only to the watch loop branch (interval > 0).
```

## Test Matrix

| Scenario | interval | PATH | Expected exit | Proves |
|----------|----------|------|---------------|--------|
| TC-415: watch survives error | 1 | "" | 124 (timeout) | loop continues |
| TC-409: one-shot still exits | 0 | "" | 0 | one-shot unchanged |
| TC-411: override dry run | 0 | "" | 0 | regression check |

## Work Procedure

1. Run `ctest3` to confirm TC-415 fails (red phase).
2. Open `src/commands.rs`; locate watch loop error arm (≈ line 605-609).
3. Replace `return result;` with the 3-field fix comment + empty arm (no return).
4. Add `// Fix(issue-415)` comment immediately above the Err arm.
5. Run `ctest3` — TC-415 must now pass; TC-409, TC-411, TC-413 must still pass.
6. Confirm no other test regressions.
7. Update task status to ✅ (Completed).

## Acceptance Criteria

- TC-415 `tc415_watch_loop_continues_after_install_error` passes.
- TC-409, TC-411, TC-413 still pass (one-shot and override behavior unchanged).
- `ctest3` exits clean (zero failures, zero clippy warnings).
- Fix comment (3 fields) present in source at the corrected Err arm.

## Validation Checklist

Desired answer for every question is YES.

- [ ] Does TC-415 pass after the fix?
- [ ] Does TC-409 still pass (one-shot behavior unchanged)?
- [ ] Does `ctest3` exit 0 (all tests + clippy clean)?
- [ ] Is the 3-field `// Fix(issue-415)` comment present in `src/commands.rs`?
- [ ] Does the watch loop Err arm NOT contain `return` or `?` after the fix?

## Validation Procedure

### Measurements

**M1 — TC-415 passes**
Command: `cargo nextest run tc415 --all-features 2>&1 | tail -5`
Before: FAILED (exit code 2 ≠ 124). Expected: PASSED. Deviation: any FAILED line.

**M2 — No return in watch loop Err arm**
Command: `grep -n "return result" src/commands.rs`
Before: line ≈ 609 contains `return result`. Expected: zero matches. Deviation: any match.

**M3 — Full suite clean**
Command: `ctest3`
Before: TC-415 FAILED. Expected: all PASSED, exit 0. Deviation: any FAILED or warning.

### Anti-faking checks

**AF1 — One-shot propagation preserved**
Check: `cargo nextest run tc409 --all-features 2>&1 | grep -c PASSED`
Expected: 1. If 0: one-shot regression introduced.

**AF2 — Fix comment present**
Check: `grep "Fix(issue-415)" src/commands.rs`
Expected: 1 match. If 0: documentation requirement not met.

## Rulebook References

- `code_design.rulebook.md` — Bug-Fixing Workflow (MRE test → fix → document → verify)
- `test_organization.rulebook.md` — 5-section test doc format; `bug_reproducer(issue-NNN)` marker
- `code_style.rulebook.md` — 3-field source fix comment format
- `codebase_hygiene.rulebook.md` — STATC quality standard for fix documentation

## Outcomes

**Fix was pre-existing in code** — the `return result` line had already been removed from
the watch loop Err arm before this session. What was missing was all accompanying
documentation: the 3-field source comment, the TC-415 failing test (which could not
pass until the code was correct), the changelog `### Fixed` entry, the FR-19 spec
resilience clause, the `commands.md` user-facing note, and the `version_guard.md`
TC-415 test case definition.

**Test pattern for watch-mode resilience:** use `/usr/bin/timeout 2 cm ... interval::1`
with `PATH=""` to force install failure. Exit code 124 (killed by timeout) = loop
survived; exit code 2 (error propagated) = bug present. This pattern is reusable for
any future watch-mode daemon resilience tests.

**All acceptance criteria met:**
- TC-415 PASSED (274/274 tests, 2.0s — killed by timeout as expected)
- Fix(issue-415) 3-field comment in `src/commands.rs` at the Err arm
- `ctest3` clean: nextest 274/274 + doc tests + clippy, zero warnings

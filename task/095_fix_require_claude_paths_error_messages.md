# Fix require_claude_paths producing identical errors for distinct failure conditions

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** 🎯 (Available)

## Goal

Fix `require_claude_paths()` in `commands.rs` so it emits distinct, actionable error messages for its two failure conditions — `HOME` unset vs `ClaudePaths::new()` returning `None` — instead of reporting "HOME environment variable not set" for both, verified by `w3 .test level::3`. (Motivated: when `ClaudePaths::new()` fails for a reason other than a missing HOME var, the user receives a misleading diagnosis and wastes time looking for a HOME issue that does not exist; Observable: each failure branch returns a distinct message referencing its actual cause; Scoped: only `require_claude_paths()` in `commands.rs` — no other changes; Testable: `cargo nextest run --test integration --features enabled -E 'test(require_paths)'`)

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_manager/src/commands.rs` — `require_claude_paths()`: distinguish the two conditions and return distinct error messages
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_manager/tests/integration/error_messages_test.rs` — add TC-320b: `HOME=""` path → error mentions "HOME"; add TC-320c: if HOME is set but ClaudePaths::new() returns None for another reason → error does NOT say "HOME environment variable not set"

## Out of Scope

- Changing `ClaudePaths::new()` itself or its logic in `claude_manager_core`
- Changing callers of `require_claude_paths()` other than fixing the error text
- Changing any other error messages in `commands.rs`

## Description

`require_claude_paths()` has two distinct failure conditions that both emit the same error message: "HOME environment variable not set". The first — `HOME` genuinely absent — is correctly described. The second — `ClaudePaths::new()` returns `None` even though `HOME` is set — is a path resolution failure unrelated to `HOME`.

When path resolution fails for a non-HOME reason, operators see a misleading message and waste time verifying an environment variable that is actually set correctly. The fix distinguishes the two cases by calling `std::env::var("HOME")` first: if `HOME` is absent, emit the existing message; if `HOME` is present but `ClaudePaths::new()` still returns `None`, emit "could not resolve Claude configuration paths (HOME is set but path resolution failed)".

No changes are needed outside `require_claude_paths()` — callers already propagate the error, and exit-code routing in `main.rs` is unaffected.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   TDD: write failing tests before implementing; confirm they fail before fixing
-   Use error_tools exclusively for error construction (no anyhow/thiserror)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note error_tools error construction patterns.
2. **Write Test Matrix** — populate all rows below before opening any test file.
3. **Write failing tests** — add TC-320b and TC-320c to `error_messages_test.rs`; confirm they fail.
4. **Read source** — read `require_claude_paths()` in `src/commands.rs`; read `ClaudePaths::new()` to understand what `None` signals there.
5. **Implement** — split the single `None` branch into two: (a) check `std::env::var("HOME")` explicitly to detect the HOME-missing case and emit "HOME environment variable not set"; (b) the fallthrough case emits "could not resolve Claude configuration paths (HOME is set but path resolution failed)".
6. **Green state** — `w3 .test level::3` must pass with zero failures and zero warnings.
7. **Submit for Validation** — trigger SUBMIT transition.
8. **Update task status** — on validation pass set ✅ in `task/readme.md`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `HOME=""` on a command that calls `require_claude_paths` | HOME-missing error branch | stderr mentions "HOME", exit 2 |
| T02 | `HOME` set but resolution fails | ClaudePaths::new() None branch | stderr does NOT say "HOME environment variable not set"; mentions path resolution |
| T03 | `HOME` set and valid | happy path | command proceeds normally, exit 0 or 2 for other reasons |

## Acceptance Criteria

-   `require_claude_paths()` has two distinct error messages — one for HOME missing, one for path resolution failure
-   The HOME-missing error still contains the word "HOME"
-   The path-resolution error does not say "HOME environment variable not set"
-   T01–T03 all pass
-   `w3 .test level::3` passes with zero failures

## Validation

### Checklist

Desired answer for every question is YES.

**Error message distinction**
- [ ] C1 — Does `require_claude_paths()` have two separate error-return branches?
- [ ] C2 — Does the HOME-missing branch produce a message containing "HOME"?
- [ ] C3 — Does the path-resolution branch produce a message that does NOT say "HOME environment variable not set"?
- [ ] C4 — Does TC-320b pass (HOME="" → error mentions "HOME")?

**Out of Scope confirmation**
- [ ] C5 — Is `ClaudePaths::new()` in `claude_manager_core` unchanged?
- [ ] C6 — Are callers of `require_claude_paths()` other than the function body itself unchanged?

### Measurements

- [ ] M1 — distinct messages: `HOME="" cm .settings.show 2>&1 | grep HOME` → non-empty (was: same message always; now: message specifically about HOME missing)

### Invariants

- [ ] I1 — test suite: `w3 .test level::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --features enabled` → 0 warnings

### Anti-faking checks

- [ ] AF1 — two branches: `grep -c "HOME environment variable not set\|path resolution" src/commands.rs` → exactly 2 (one distinct message per condition)
- [ ] AF2 — no identical errors: `grep -A2 "require_claude_paths" src/commands.rs | grep -c "HOME environment variable not set"` → exactly 1 (only the HOME-missing branch)

## Outcomes

[Added upon task completion.]

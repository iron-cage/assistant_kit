# Change ultrathink injection from prefix to suffix with `\n\n` separator

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** 🎯 (Available)

## Goal

Fix `build_claude_command()` in `module/claude_runner/src/main.rs` to append
`"\n\nultrathink"` as a suffix instead of prepending `"ultrathink "` as a prefix,
and update all dependent tests so that `w3 .test level::3` passes with zero
failures (Motivated: live feedback from `-feedback.md` shows the wrong injection
order; Observable: `clr --dry-run "hi"` produces `"hi\n\nultrathink"` not
`"ultrathink hi"`; Scoped: `main.rs` + `tests/cli_args_test.rs` only;
Testable: `w3 .test level::3` passes with zero failures).

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/src/main.rs`
  § `build_claude_command()` — change `format!("ultrathink {msg}")` →
  `format!("{msg}\n\nultrathink")` and update idempotent guard from
  `msg.starts_with("ultrathink")` → `msg.trim_end().ends_with("ultrathink")`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/src/main.rs`
  § module-level doc comment (line ~30) — update "prefixed" → "suffixed after `\n\n`"
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/tests/cli_args_test.rs`
  — update t10, t27, t33, t37, t50, t51, t52 assertions and doc comments; add
  failing test for suffix behavior before implementing fix (TDD order)

## Out of Scope

- Documentation markdown files under `docs/` (already updated by doc_tsk)
- Any `claude_runner_core` crate changes — the suffix is injected in `main.rs`
  before calling `builder.with_message()`, not inside the builder
- Any other crates (claude_manager, claude_profile, claude_assets, etc.)

## Description

`build_claude_command()` at `main.rs:343` currently produces:

```rust
format!( "ultrathink {msg}" )   // "ultrathink hello"
```

The correct behavior (confirmed via `-feedback.md` live trace) is:

```rust
format!( "{msg}\n\nultrathink" )  // "hello\n\nultrathink"
```

This places the ultrathink directive *after* the task description, separated by
two newlines — matching the conversational pattern "state task, then direct
thinking mode." The idempotent guard must also change from `starts_with` (which
anchors at the front) to `trim_end().ends_with` (which anchors at the back).

Seven existing tests assert the prefix form (`"ultrathink {msg}"`) and will
fail after the fix. These must be updated to assert the suffix form. One new
failing test (T58) must be written first to TDD-validate the suffix behavior,
then the fix is applied, and all tests must pass.

The docs were already updated by doc_tsk — this task covers only code and tests.

## Requirements

- All work must strictly adhere to all applicable rulebooks
  (discover via `kbase .rulebooks`)
- TDD order: write T58 failing test → implement fix → verify T58 passes →
  update t10/t27/t33/t37/t50/t51/t52 assertions → full suite green
- Idempotent guard must anchor at end: `msg.trim_end().ends_with("ultrathink")`
- No change to `cli_args_test.rs` module-level doc comment index (T58 gets a
  new row; existing rows updated to say "suffix")
- Fix documentation comment (3-field format: `Fix(issue-ultrathink-suffix)`,
  `Root cause`, `Pitfall`) must be added at the injection site in `main.rs`

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note code_style and
   test_organization constraints on function length, fix comment format,
   and test doc format.
2. **Read source** — Read `src/main.rs` in full to understand current
   `build_claude_command()` at lines 332-348 and module doc at line 30.
3. **Read tests** — Read `tests/cli_args_test.rs` around t50/t51/t52 (lines
   750-838) and t10/t27/t33/t37 (lines 200-527) to understand assertion forms.
4. **Write T58 failing test** — Add `t58_default_message_gets_ultrathink_suffix()`
   after t57 in `cli_args_test.rs`. Assert that dry-run output contains
   `"hello\n\nultrathink"` (or equivalent suffix check). Confirm the test fails
   before the fix. Also add T58 row to the module doc comment index.
5. **Implement fix** — In `main.rs`:
   a. Change `msg.starts_with("ultrathink")` → `msg.trim_end().ends_with("ultrathink")`
   b. Change `format!("ultrathink {msg}")` → `format!("{msg}\n\nultrathink")`
   c. Add fix comment at the injection site
   d. Update module doc comment line 30: "prefixed with `\"ultrathink \"`" →
      "suffixed after `\"\n\n\"` with `\"ultrathink\"`"
6. **Update affected tests** — Update assertions in t10, t27, t33, t37, t50,
   t51, t52 to use suffix form. Update doc comments in those tests to say
   "suffix" / "appended" instead of "prefix" / "prepended".
7. **Green state** — `w3 .test level::3` must pass with zero failures and
   zero warnings.
8. **Walk Validation Checklist** — every item YES.
9. **Update task status** — set ✅ in `task/readme.md`, move file to
   `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T58 | `clr --dry-run "hello"` | default (ultrathink on) | output contains `"hello"` before `"ultrathink"`; does NOT contain `"ultrathink hello"` |
| t50 (updated) | `clr --dry-run "hello"` | default | output contains suffix `"\n\nultrathink"` (not prefix) |
| t51 (updated) | `clr --dry-run --no-ultrathink "hello"` | ultrathink off | output does NOT contain `"ultrathink"` at all |
| t52 (updated) | `clr --dry-run` with message ending in `"ultrathink"` | idempotent guard | no double-suffix (`"ultrathink\n\nultrathink"` absent) |
| t10 (updated) | `clr --dry-run --dir /tmp --model ... "fix it"` | combined flags | output contains `"fix it"` before `"ultrathink"`; `"ultrathink fix it"` absent |
| t27 (updated) | `clr --dry-run -- --not-a-flag` | `--` separator | output contains `"--not-a-flag"` before `"ultrathink"` |
| t33 (updated) | `clr --dry-run "hello" --dir /tmp "world"` | interleaved positional | output contains `"hello world"` before `"ultrathink"` |
| t37 (updated) | `clr --dry-run "Fix" "the" "bug" "now"` | multiple positional | output contains `"Fix the bug now"` before `"ultrathink"` |

## Acceptance Criteria

- `clr --dry-run "hello"` output contains `"ultrathink"` as suffix, NOT as prefix;
  `"ultrathink hello"` is absent
- `clr --dry-run --no-ultrathink "hello"` output does NOT contain `"ultrathink"`
- `clr --dry-run` with a message already ending in `"ultrathink"` does not produce double suffix
- T58 test exists in `tests/cli_args_test.rs` and passes
- All seven previously failing tests (t10, t27, t33, t37, t50, t51, t52) pass with updated assertions
- `main.rs` fix comment at the injection site uses 3-field format: `Fix(issue-ultrathink-suffix)`
- `w3 .test level::3` passes with zero failures

## Validation

### Checklist

Desired answer for every question is YES.

**Injection logic (`src/main.rs`)**
- [ ] Does `build_claude_command()` use `format!("{msg}\n\nultrathink")` (not `format!("ultrathink {msg}")`)?
- [ ] Does the idempotent guard use `msg.trim_end().ends_with("ultrathink")` (not `starts_with`)?
- [ ] Is there a `Fix(issue-ultrathink-suffix)` comment at the injection site with Root cause and Pitfall?
- [ ] Is the module doc (line ~30) updated to say "suffixed" instead of "prefixed"?

**Tests (`tests/cli_args_test.rs`)**
- [ ] Does T58 exist and assert suffix behavior (`"hello\n\nultrathink"` or equivalent)?
- [ ] Are t10/t27/t33/t37/t50/t51/t52 assertions updated to suffix form (no `"ultrathink {msg}"` form remains)?
- [ ] Does t52 test a message that ends with `"ultrathink"` (not starts with)?

**Out of Scope confirmation**
- [ ] Are all `docs/` markdown files unchanged (no edits to `.md` files)?
- [ ] Is `claude_runner_core` unchanged?

### Measurements

**M1 — Suffix assertion in T58**
Command: `grep -c "t58_default_message_gets_ultrathink_suffix" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/tests/cli_args_test.rs`
Before: 0 (test doesn't exist). Expected: `1`. Deviation: test not created.

**M2 — No remaining prefix assertions**
Command: `grep -c "ultrathink hello\"\|ultrathink fix it\"\|ultrathink Fix the\"\|ultrathink --not" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/tests/cli_args_test.rs`
Before: multiple matches (prefix form). Expected: `0`. Deviation: stale prefix assertions remain.

**M3 — Injection format in main.rs**
Command: `grep -c 'format!.*{msg}.*ultrathink' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/src/main.rs`
Before: 0 (current form is `ultrathink {msg}`). Expected: `1`. Deviation: format not changed.

### Invariants

- [ ] I1 — test suite: `w3 .test level::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --all-features` → 0 warnings

### Anti-faking checks

**AF1 — Guard uses ends_with not starts_with**
Check: `grep -c 'starts_with.*ultrathink' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/src/main.rs`
Expected: `0`. Why: catches the case where the guard is left unchanged while only the format string changes.

**AF2 — Prefix format string absent**
Check: `grep -c '"ultrathink {msg}"' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/src/main.rs`
Expected: `0`. Why: confirms the old format string was replaced, not just commented out.

## Outcomes

[Added upon task completion.]

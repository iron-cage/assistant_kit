# Implement ultrathink default message prefix in `clr` binary

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** ✅ (Complete)

## Goal

Implement default-on `"ultrathink "` message prefix and `--no-ultrathink` opt-out flag in `clr`, then update all stale test assertions and add 7 new tests covering the new behavior (Motivated: ultrathink prefix activates Claude's extended thinking mode for every automation invocation without requiring user intervention — currently missing, so every automated call runs without extended reasoning; Observable: `clr --dry-run "hello"` outputs a command containing `"ultrathink hello"`, `clr --dry-run --no-ultrathink "hello"` outputs verbatim `"hello"`, and `cargo nextest run -p claude_runner` reports 0 failures; Scoped: `module/claude_runner/src/main.rs` + 2 test files only — no other crates, no documentation edits; Testable: `ctest3` passes with zero failures and zero clippy warnings).

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/src/main.rs`
  - Add `no_ultrathink: bool` field to `CliArgs` struct (default false = ultrathink ON)
  - Add `"--no-ultrathink"` parse arm to `parse_args()` (after `"--no-skip-permissions"`)
  - Add `--no-ultrathink` help line to `print_help()` (column-aligned with other options)
  - Replace bare `builder.with_message(msg.clone())` with conditional ultrathink prefix block in `build_claude_command()`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/tests/cli_args_test.rs`
  - Fix 5 stale message assertions: T01 (`"hello"`), T10 (`"fix it"`), T27 (`"--not-a-flag"`), T33 (`"hello world"`), T37 (`"Fix the bug now"`) — prefix each with `"ultrathink "`
  - Update T16: add `"--no-ultrathink"` to the expected options list
  - Update T48: add `"--no-ultrathink"` to invocation args (preserves its original `--no-skip-permissions --new-session` focus while suppressing ultrathink side-effect)
  - Add T50–T53: default-on, opt-out, idempotent guard, help listing
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/tests/dry_run_test.rs`
  - Fix 3 stale message assertions: `message_appears_in_command_quoted`, `combined_flags_all_appear`, `message_param_appears_in_command`
  - Add 3 new tests: `ultrathink_prefix_default_on`, `no_ultrathink_flag_suppresses_prefix`, `ultrathink_idempotent_guard`

## Out of Scope

- Documentation updates (already completed by doc_pln: params.md, commands.md, parameter_groups.md, parameter_interactions.md, invariant/001_default_flags.md, feature/001_runner_tool.md, dictionary.md, types.md, all testing/ docs)
- Any changes to `claude_runner_core`, `claude_storage`, or any other crate
- Changes to `tests/manual/readme.md` beyond the automated test scope
- Adding new test files — work goes into the two existing test files only

## Description

`clr` currently sends every message verbatim to the `claude` subprocess. The ultrathink prefix (`"ultrathink "`) is a message-level instruction that activates Claude's extended thinking mode; it cannot be a CLI flag because it is consumed by Claude's prompt parser (part of the user turn), not by Claude Code CLI argument parsing.

The invariant at `docs/invariant/001_default_flags.md` documents the required default-on behavior: all four defaults (`-c`, `--dangerously-skip-permissions`, `--chrome`, and `"ultrathink "` prefix) must be injected automatically. The implementation plan at `-plan/002_ultrathink_default_feature.plan.md` specifies exact code changes and test updates.

The core change is a three-line conditional in `build_claude_command()`: if `no_ultrathink` is false and the message does not already start with `"ultrathink"`, prepend `"ultrathink "`. The idempotent guard uses `starts_with("ultrathink")` without a trailing space to catch both `"ultrathink "` and `"ultrathink\n"` edge cases. Nine existing tests assert exact message strings that will fail once the prefix is active; each must be updated. Seven new tests cover the new behavioral surface: default-on, opt-out, idempotent guard, no-message path, and help listing.

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- `no_ultrathink: bool` must use `#[derive(Default)]` — no explicit `false` literal in struct definition
- Idempotent guard: `msg.starts_with("ultrathink")` — no trailing space, case-sensitive by design
- Help line column alignment: `--no-ultrathink` description must align at the same column as all other option descriptions (T49 regression guard)
- Parse arm must appear after `"--no-skip-permissions"` to maintain logical grouping of opt-out flags
- No new test files — all tests go into the two existing test files
- No `#[ignore]`, no `todo!()`, no `unimplemented!()` — all tests must have real assertions
- Bug-reproducer 5-section doc comment is NOT required for stale-test updates (these are specification changes, not bug fixes)
- Code style: 2-space indent, spaced brackets — no `cargo fmt`

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note `code_style.rulebook.md` formatting constraints (2-space indent, spaced brackets) and `test_organization.rulebook.md` for test doc comment conventions.
2. **Read specification** — Read `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/invariant/001_default_flags.md` and `-plan/002_ultrathink_default_feature.plan.md` Phase 1 and Phase 2 steps as the authoritative source for exact code to write.
3. **Read main.rs** — Read `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/src/main.rs` to confirm exact insertion points for each of the 4 edits. Verify `with_message()` is called exactly once.
4. **Grep stale assertions** — Run `grep -n '"fix it"\|"hello"\|"Hello there"\|"hello world"\|"Fix the bug now"\|"--not-a-flag"' module/claude_runner/tests/cli_args_test.rs module/claude_runner/tests/dry_run_test.rs` to locate all 9 stale lines.
5. **Write failing tests (TDD)** — Add T50–T53 to `cli_args_test.rs` and 3 new tests to `dry_run_test.rs`. These will fail at runtime because `--no-ultrathink` is not yet recognized by the binary. Confirm the tests compile.
6. **Implement main.rs** — Apply the 4 edits in this order: (a) `CliArgs` struct field, (b) `parse_args()` arm, (c) `print_help()` line, (d) `build_claude_command()` message block. Build after each edit: `RUSTFLAGS="-D warnings" cargo build -p claude_runner`.
7. **Fix stale assertions** — Update all 9 stale message assertions (T01, T10, T27, T33, T37 in cli_args_test.rs; `message_appears_in_command_quoted`, `combined_flags_all_appear`, `message_param_appears_in_command` in dry_run_test.rs) to include the `"ultrathink "` prefix. Update T16 and T48 per spec.
8. **Verify per-crate green** — `RUSTFLAGS="-D warnings" cargo nextest run -p claude_runner`. Must show 0 failures before proceeding.
9. **Walk Validation Checklist** — check every item. Every answer must be YES.
10. **Run full workspace validation** — `ctest3`. Fix any failures or warnings before declaring done.
11. **Update task status** — Set ✅ in `task/readme.md`, recalculate advisability to 0 (Priority=0), move file to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|----------------|-------------------|-------------------|
| T01 | `--dry-run "hello"` | no_ultrathink=false (default) | Output contains `"ultrathink hello"` |
| T02 | `--dry-run --no-ultrathink "hello"` | no_ultrathink=true | Output contains `"hello"`, NOT `"ultrathink hello"` |
| T03 | `--dry-run "ultrathink fix the bug"` | no_ultrathink=false (default) | Output contains `"ultrathink fix the bug"` exactly (no double-prefix) |
| T04 | `--dry-run` (no message) | no_ultrathink=false | Output does NOT contain `"ultrathink"` anywhere |
| T05 | `--help` | any | Output contains `--no-ultrathink` |
| T06 | `--dry-run "hello world"` (dry_run_test) | no_ultrathink=false | Dry-run output contains `"ultrathink hello world"` |
| T07 | `--dry-run --no-ultrathink "fix the bug"` (dry_run_test) | no_ultrathink=true | Dry-run output contains `"fix the bug"` verbatim, no ultrathink |
| T08 | `--dry-run "ultrathink fix it"` (dry_run_test) | no_ultrathink=false | Dry-run output contains `"ultrathink fix it"`, NOT `"ultrathink ultrathink fix it"` |

## Acceptance Criteria

- `CliArgs` struct in `main.rs` contains `no_ultrathink: bool` (derived default = false)
- `parse_args()` recognizes `"--no-ultrathink"` and sets `parsed.no_ultrathink = true`
- `build_claude_command()` contains the idempotent guard: `cli.no_ultrathink || msg.starts_with("ultrathink")`
- `print_help()` lists `--no-ultrathink` with description aligned at the same column as `--no-skip-permissions`
- All 5 stale message assertions in `cli_args_test.rs` (T01, T10, T27, T33, T37) now include `"ultrathink "` prefix
- T16 includes `"--no-ultrathink"` in its expected options slice
- T48 passes `"--no-ultrathink"` in its invocation args
- All 3 stale message assertions in `dry_run_test.rs` now include `"ultrathink "` prefix
- Functions T50, T51, T52, T53 exist in `cli_args_test.rs` with non-trivial assertions
- Functions `ultrathink_prefix_default_on`, `no_ultrathink_flag_suppresses_prefix`, `ultrathink_idempotent_guard` exist in `dry_run_test.rs` with non-trivial assertions
- `cargo nextest run -p claude_runner` reports 0 failures
- `ctest3` (full workspace Level 3) reports 0 failures and 0 clippy warnings

## Validation

### Checklist

Desired answer for every question is YES.

**`main.rs` — struct and parsing**
- [ ] Does `CliArgs` contain `no_ultrathink: bool` without an explicit `false` default literal?
- [ ] Does `parse_args()` match `"--no-ultrathink"` and set `parsed.no_ultrathink = true`?
- [ ] Does the parse arm appear after `"--no-skip-permissions"`?

**`main.rs` — help output**
- [ ] Does `print_help()` print a line containing `--no-ultrathink`?
- [ ] Is the description column of `--no-ultrathink` aligned with other option descriptions?

**`main.rs` — message transformation**
- [ ] Does `build_claude_command()` prepend `"ultrathink "` when `!cli.no_ultrathink && !msg.starts_with("ultrathink")`?
- [ ] Is the idempotent guard `msg.starts_with("ultrathink")` (no trailing space)?
- [ ] Is `with_message()` called exactly once (only inside the `if let Some(ref msg)` block)?
- [ ] Does `cargo build -p claude_runner` produce zero errors and zero warnings?

**`cli_args_test.rs` — stale assertions**
- [ ] Does T01 assert `"ultrathink hello"` (not bare `"hello"`)?
- [ ] Does T10 assert `"ultrathink fix it"`?
- [ ] Does T27 assert `"ultrathink --not-a-flag"`?
- [ ] Does T33 assert `"ultrathink hello world"`?
- [ ] Does T37 assert `"ultrathink Fix the bug now"`?
- [ ] Does T16 include `"--no-ultrathink"` in its expected options?
- [ ] Does T48 pass `"--no-ultrathink"` in its invocation args?

**`cli_args_test.rs` — new tests**
- [ ] Does T50 (`t50_ultrathink_prefix_default_on`) assert output contains `"ultrathink hello"`?
- [ ] Does T51 (`t51_no_ultrathink_suppresses_prefix`) assert verbatim message and absence of `"ultrathink"`?
- [ ] Does T52 (`t52_ultrathink_idempotent_guard`) assert no double-prefix?
- [ ] Does T53 (`t53_no_ultrathink_flag_in_help`) assert `--no-ultrathink` in help output?

**`dry_run_test.rs` — stale assertions**
- [ ] Does `message_appears_in_command_quoted` assert `"ultrathink hello world"`?
- [ ] Does `combined_flags_all_appear` assert `"ultrathink fix it"`?
- [ ] Does `message_param_appears_in_command` assert `"ultrathink Hello there"`?

**`dry_run_test.rs` — new tests**
- [ ] Does `ultrathink_prefix_default_on` assert output contains `"ultrathink fix the bug"`?
- [ ] Does `no_ultrathink_flag_suppresses_prefix` assert verbatim message and absence of ultrathink prefix?
- [ ] Does `ultrathink_idempotent_guard` assert no double-prefix?

**Full suite**
- [ ] Does `cargo nextest run -p claude_runner` report 0 failures?
- [ ] Does `ctest3` report 0 failures and 0 clippy warnings?

**Out of Scope confirmation**
- [ ] Are documentation files in `docs/` unchanged from the state after doc_pln?
- [ ] Are `claude_runner_core`, `claude_storage`, and all other crates unchanged?
- [ ] Are there zero `#[ignore]`, `todo!()`, or `unimplemented!()` in the modified test files?

### Measurements

**M1 — Binary compiles clean**
Command: `RUSTFLAGS="-D warnings" cargo build -p claude_runner 2>&1 | tail -3`
Before: clean build (no ultrathink). Expected after: `Finished` line, 0 errors, 0 warnings. Deviation: any error or warning = must fix before proceeding.

**M2 — Dry-run shows ultrathink prefix by default**
Command: `./target/debug/clr --dry-run "hello" 2>/dev/null`
Before: output contained `"hello"`. Expected after: output contains `"ultrathink hello"`. Deviation: missing prefix = transformation not wired to `with_message()`.

**M3 — `--no-ultrathink` suppresses prefix**
Command: `./target/debug/clr --dry-run --no-ultrathink "hello" 2>/dev/null`
Before: flag did not exist. Expected after: output contains `"hello"` (verbatim), NOT `"ultrathink hello"`. Deviation: prefix still appears = opt-out flag not wired.

**M4 — Idempotent guard prevents double-prefix**
Command: `./target/debug/clr --dry-run "ultrathink fix it" 2>/dev/null`
Before: N/A (transformation didn't exist). Expected after: output contains `"ultrathink fix it"` exactly, NOT `"ultrathink ultrathink fix it"`. Deviation: double-prefix = guard missing.

**M5 — Per-crate test suite green**
Command: `RUSTFLAGS="-D warnings" cargo nextest run -p claude_runner 2>&1 | tail -5`
Before: 9 tests would fail after Phase 1 binary change. Expected after: 0 failures. Deviation: any failure = stale assertion or broken new test.

**M6 — Full workspace ctest3 green**
Command: `ctest3 2>&1 | tail -10`
Before: passing (before this task). Expected after: still passing with 0 failures, 0 warnings. Deviation: regression in another crate = unexpected side effect to investigate.

### Anti-faking checks

**AF1 — Transformation present in `build_claude_command`**
Check: `grep -n "ultrathink" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/src/main.rs`
Expected: ≥2 matches (the guard `starts_with("ultrathink")` and the `format!("ultrathink {msg}")` call). Zero = transformation not implemented. One only = partial implementation.

**AF2 — `with_message` called exactly once**
Check: `grep -c "with_message" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/src/main.rs`
Expected: 1. More than 1 = transformation split across code paths, violating the single-site rule.

**AF3 — New tests have real assertions**
Check: `grep -A8 "fn t50_\|fn t51_\|fn t52_\|fn t53_\|fn ultrathink" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/tests/cli_args_test.rs /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/tests/dry_run_test.rs | grep -c "assert!"`
Expected: ≥14 (minimum 2 asserts per test × 7 tests). Fewer = test bodies empty or trivial.

**AF4 — No stale bare message assertions without `--no-ultrathink` context**
Check: `grep -n '"fix it"\|"Hello there"\|"hello world"\|"Fix the bug now"\|"--not-a-flag"' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/tests/cli_args_test.rs /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/tests/dry_run_test.rs`
Expected: any surviving match must be inside a function that also invokes `--no-ultrathink` (T48, T51, `no_ultrathink_flag_suppresses_prefix`). A bare match in a default-mode test = stale assertion not updated.

**AF5 — No `#[ignore]` in test files**
Check: `grep -n "#\[ignore\]" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/tests/cli_args_test.rs /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/tests/dry_run_test.rs`
Expected: zero matches. Any match = a test was silenced instead of fixed.

## Outcomes

- **Completed:** 2026-04-11
- **Executor:** claude-sonnet-4-6
- **Files Modified:** `src/main.rs` (+16 LOC: struct field, parse arm, help line, transformation block, `#[allow]`), `tests/cli_args_test.rs` (+100 LOC: 4 corner cases entries, T50-T53, 7 stale assertion fixes), `tests/dry_run_test.rs` (+65 LOC: 3 corner cases entries, `run_cli` helper, 3 new tests, 3 stale assertion fixes), `changelog.md` (+9 LOC)
- **Test Count Change:** 112 → 119 (+7 new tests; 9 stale assertions updated)
- **Workspace Validation:** 1559/1559 tests pass; 131 doc tests pass; clippy clean (0 errors, exit 0)
- **Key Decisions:**
  - `#[allow(clippy::too_many_lines)]` added to `parse_args()` — function reaches 101 lines after adding the new arm; the only extractable candidates had no complex logic; this is explicitly anticipated by the existing `parse_token_limit` doc comment
  - `run_cli` helper duplicated in `dry_run_test.rs` — needed for `no_ultrathink_flag_suppresses_prefix` which must use raw output (not panicking `run_dry`) before Phase 3; consistent with `cli_args_test.rs` naming
  - T52 / `ultrathink_idempotent_guard` pass before Phase 3 by design — idempotent guard tests inherently satisfy their assertions in both "no prefix" and "guarded prefix" states; not a TDD violation

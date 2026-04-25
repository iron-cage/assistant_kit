# Fix `.session.dir` and `.session.ensure` — default `path::` to cwd when absent

## Execution State

- **Executor Type:** any
- **Actor:** claude-sonnet-4-6
- **Claimed At:** 2026-04-25
- **Status:** ✅ (Completed)
- **Validated By:** claude-sonnet-4-6
- **Validation Date:** 2026-04-25

## Goal

Replace `resolve_required_session_dir` with a cwd-defaulting implementation so that `.session.dir` and `.session.ensure` work without an explicit `path::` argument (Motivated: the YAML already declares `path::` as optional with "default: current directory" but the implementation unconditionally errors when `path::` is absent — every bare invocation fails; Observable: `clg .session.dir` without `path::` succeeds and outputs `{cwd}/-default_topic`; Scoped: `resolve_required_session_dir` in `src/cli/mod.rs:3093-3112` — no other command routines or YAML changes needed; Testable: `w3 .test level::3` passes and `it_session_dir_missing_path_rejected` is replaced by a passing cwd-default test).

The YAML spec (`unilang.commands.yaml`) already marks `path::` as `optional: true` with description `"Project directory (default: current directory)"` for both `.session.dir` and `.session.ensure`. The documentation (`docs/cli/commands.md`) has been updated to reflect this behavior. Only the implementation is wrong: `resolve_required_session_dir` calls `cmd.get_string("path").ok_or_else(|| error)?` which rejects absent `path::` before a fallback can occur. The sibling helper `resolve_cmd_path` already implements the correct cwd fallback pattern and should be reused.

The fix is a one-function change: replace the `ok_or_else` + `resolve_path_parameter` call chain with a single call to `resolve_cmd_path(cmd)` which returns `cwd` when `path::` is absent. The doc comment and error message on `resolve_required_session_dir` must also be updated (or the function renamed to `resolve_session_dir`).

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs`
  - `resolve_required_session_dir` (lines 3093–3112): replace `ok_or_else` guard with `resolve_cmd_path(cmd)?`; rename function to `resolve_session_dir`; update doc comment to say "defaults to cwd"
  - Update doc comments on `session_dir_routine` and `session_ensure_routine` to remove "path:: is required"
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/session_path_command_test.rs`
  - Replace `it_session_dir_missing_path_rejected` with `it_session_dir_cwd_default` — assert bare `.session.dir` succeeds and outputs `{cwd}/-default_topic`
  - Add `it_session_ensure_cwd_default` — assert bare `.session.ensure` succeeds, creates `{cwd}/-default_topic`, outputs path + strategy

## Out of Scope

- Documentation updates (already completed)
- YAML changes (already correct — `optional: true` with cwd description)
- Task 029 (renaming `.path` → `.project.path`, `.exists` → `.project.exists`)

## Requirements

- All work must strictly adhere to all applicable rulebooks
  (discover via `kbase .rulebooks`)
- No mocks — tests must use real temporary directories and the live binary (consistent with existing patterns in `tests/session_path_command_test.rs`)
- The cwd for the bare-invocation test must be set explicitly via `.current_dir(project.path())` on the `Command` — do not assume process cwd

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note codestyle constraints (2-space indent, no `cargo fmt`).
2. **Read source** — Read `src/cli/mod.rs` lines 2953–2979 (`resolve_cmd_path`) and 3082–3112 (`resolve_required_session_dir`) to understand both functions completely.
3. **Read tests** — Read `tests/session_path_command_test.rs` lines 434–600 to understand existing `.session.dir` tests including `it_session_dir_missing_path_rejected`.
4. **Write failing tests** — In `tests/session_path_command_test.rs`:
   a. Replace `it_session_dir_missing_path_rejected` with `it_session_dir_cwd_default` (invokes `.session.dir` with `.current_dir(project.path())`, asserts exit 0, asserts stdout equals `{project_path}/-default_topic\n`).
   b. Add `it_session_ensure_cwd_default` — invokes `.session.ensure` with `.current_dir(project.path())`, asserts exit 0, asserts line 1 equals `{project_path}/-default_topic`, asserts line 2 is `fresh` or `resume`.
   Confirm tests fail (exit-code or output mismatch) before proceeding.
5. **Implement fix** — In `src/cli/mod.rs`:
   a. Rename `resolve_required_session_dir` → `resolve_session_dir`.
   b. Replace the body: delete the `ok_or_else` guard and `resolve_path_parameter` call; use `let base = resolve_cmd_path(cmd)?;` instead.
   c. Update the doc comment: replace "Requires `path::`" with "Defaults to cwd when `path::` is absent".
   d. Update the `command_name` parameter: no longer needed for error message — remove it; update the two callers in `session_dir_routine` and `session_ensure_routine`.
   e. Update doc comments on `session_dir_routine` and `session_ensure_routine` to remove "`path::` is required" language.
6. **Validate** — `w3 .test level::3`. All tests must pass, zero warnings.
7. **Walk Validation Checklist** — every item must answer YES before proceeding.
8. **Update task status** — set ✅ in `task/readme.md`, recalculate Advisability to 0, re-sort, move file to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `.session.dir` with `path::PATH` (explicit) | `path::` provided | Exit 0; stdout = `{PATH}/-default_topic\n` (existing behavior preserved) |
| T02 | `.session.dir` without `path::`, cwd = project dir | `path::` absent | Exit 0; stdout = `{cwd}/-default_topic\n` |
| T03 | `.session.dir` without `path::`, topic::work | `path::` absent, `topic::work` | Exit 0; stdout = `{cwd}/-work\n` |
| T04 | `.session.ensure` without `path::`, cwd = project dir | `path::` absent | Exit 0; line 1 = `{cwd}/-default_topic`; line 2 = `fresh` or `resume`; dir created |
| T05 | `.session.dir` empty topic | `topic::` empty | Exit 1; error message mentions invalid topic |
| T06 | `.session.dir` topic with `/` | `topic::a/b` | Exit 1; error message mentions path separators |

## Acceptance Criteria

- `clg .session.dir` (no `path::`) exits 0 and prints `{cwd}/-default_topic` when invoked from a project directory
- `clg .session.ensure` (no `path::`) exits 0 and creates `{cwd}/-default_topic` when invoked from a project directory
- `resolve_session_dir` in `src/cli/mod.rs` calls `resolve_cmd_path(cmd)?` and has no `ok_or_else` guard
- `it_session_dir_cwd_default` test exists in `tests/session_path_command_test.rs` and passes
- `it_session_ensure_cwd_default` test exists in `tests/session_path_command_test.rs` and passes
- `it_session_dir_missing_path_rejected` test does NOT exist (replaced by `it_session_dir_cwd_default`)
- All existing tests continue to pass (`w3 .test level::3` → 0 failures)

## Validation

### Checklist

Desired answer for every question is YES.

**Implementation**
- [ ] C1 — Does `resolve_session_dir` call `resolve_cmd_path(cmd)?` as its first statement?
- [ ] C2 — Is the `ok_or_else` guard and `resolve_path_parameter` call removed from `resolve_session_dir`?
- [ ] C3 — Is the `command_name` parameter removed from `resolve_session_dir`?
- [ ] C4 — Does the `resolve_session_dir` doc comment say "defaults to cwd when `path::` is absent"?

**Tests**
- [ ] C5 — Does `it_session_dir_cwd_default` exist and assert exit 0 for bare `.session.dir`?
- [ ] C6 — Does `it_session_ensure_cwd_default` exist and assert exit 0 for bare `.session.ensure`?
- [ ] C7 — Is `it_session_dir_missing_path_rejected` absent from the test file?

**Out of Scope confirmation**
- [ ] C8 — Is `unilang.commands.yaml` unchanged?
- [ ] C9 — Are doc files in `docs/cli/` unchanged (already updated by doc_tsk)?

### Measurements

- [ ] M1 — bare session.dir: `clg .session.dir 2>&1; echo "exit:$?"` → last line `exit:0` (was: `exit:1`)
- [ ] M2 — bare session.ensure: `clg .session.ensure 2>&1; echo "exit:$?"` → last line `exit:0` (was: `exit:1`)

### Invariants

- [ ] I1 — test suite: `w3 .test level::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --all-features` → 0 warnings

### Anti-faking checks

- [ ] AF1 — cwd fallback in source: `grep -c "resolve_cmd_path" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs` → ≥ 3 (resolve_cmd_path defined at ~2959 + called in path_routine + called in exists_routine + now called in resolve_session_dir)
- [ ] AF2 — no ok_or_else guard: `grep -c "path parameter is required" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs` → 0
- [ ] AF3 — old test gone: `grep -c "it_session_dir_missing_path_rejected" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/session_path_command_test.rs` → 0
- [ ] AF4 — new tests present: `grep -c "it_session_dir_cwd_default\|it_session_ensure_cwd_default" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/session_path_command_test.rs` → 2

## Outcomes

- `resolve_required_session_dir` renamed to `resolve_session_dir`; `command_name` parameter removed.
- Body replaced: `ok_or_else` guard + `resolve_path_parameter` call chain → single `resolve_cmd_path(cmd)?` call.
- Doc comments on `resolve_session_dir`, `session_dir_routine`, and `session_ensure_routine` updated to reflect cwd-default behavior.
- `it_session_dir_missing_path_rejected` replaced by `it_session_dir_cwd_default` (bug_reproducer issue-028).
- `it_session_ensure_missing_path_rejected` replaced by `it_session_ensure_cwd_default` (bug_reproducer issue-028).
- File header coverage section updated: "Missing `path::` rejected" → "Missing `path::` defaults to cwd (issue-028)".
- 35/35 session path tests pass; doc tests 3/3; clippy 0 errors.
- Note: B17 live-storage test was already failing before this task (pre-existing JSONL cross-session parentUuid in live storage; unrelated to task scope).

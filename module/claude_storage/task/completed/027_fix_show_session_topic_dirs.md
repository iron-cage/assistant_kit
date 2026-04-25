# Fix `.show session_id::` — search topic project directories

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** 2026-04-25
- **Status:** ✅ (Complete)

## Goal

Implement scope-aware session lookup in Case 2 of `.show` (`session_id::` without `project::`) so that sessions stored in topic project directories (`--commit`, `--default-topic`, etc.) are found when searching from the corresponding cwd (Motivated: `clg .show session_id::9e9bc9ea` returns "Session not found: 9e9bc9ea" even though `clg .projects` shows that session under the current project — the command is broken for any real session that happened to be recorded against a topic directory; Observable: `clg .show session_id::9e9bc9ea` succeeds and displays session content when the session lives in a `--commit` topic dir for the cwd; Scoped: `show_session_in_cwd_impl` in `src/cli/mod.rs` plus `scope` parameter in `unilang.commands.yaml` — all other `.show` cases and all other commands are untouched; Testable: `w3 .test level::3` passes and a new `bug_reproducer` test verifies a session in a `--commit` topic dir is found by `.show session_id::`).

The root cause is that `show_session_in_cwd_impl` calls `storage.load_project_for_cwd()`, which does an exact match on the encoded base path and returns at most one project directory. Sessions recorded when the user's cwd was in a `-commit` or `-default_topic` topic directory are stored under separate project dirs with `--commit` or `--default-topic` suffixes in storage. These are invisible to `load_project_for_cwd()`. By contrast, `projects_routine` uses `list_projects()` filtered by scope predicate and therefore sees all topic dirs — which is why `.projects` shows the session but `.show session_id::` cannot find it.

The fix replaces `load_project_for_cwd()` with a `list_projects()` scan filtered by the scope::local predicate (`dir_name == eb || dir_name.starts_with(&format!("{eb}--"))`), iterating all matching projects until the session is found or all are exhausted.

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs`
  - `show_session_in_cwd_impl`: replace `storage.load_project_for_cwd()` with `storage.list_projects()` filtered by scope::local predicate; iterate matching projects; return first successful `format_session_output`; fall through to "Session not found" error
  - Add `Fix(issue-036)` source comment documenting root cause and pitfall
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/unilang.commands.yaml`
  - Add `scope` parameter to `.show` command (kind: String, optional, default: `local`)
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/`
  - Add `bug_reproducer` test: create session in `--commit` topic dir, assert `.show session_id::` from cwd finds it

## Out of Scope

- `path::` parameter support for `.show` — scope anchor override is a separate future enhancement
- scope::around, scope::global, scope::relevant, scope::under for `.show` Case 2 — only scope::local implemented by this task
- Changing Case 1 (no session_id), Case 3 (project:: only), Case 4 (both parameters) behavior
- Task 025 (`decode_project_display` fix) and Task 026 (storage key parsing refactor)
- Wiring the `scope::` user parameter through to the search (the scope parameter is registered in YAML; implementing runtime override is the next step)

## Requirements

- All work must strictly adhere to all applicable rulebooks
  (discover via `kbase .rulebooks`)
- No mocks — the bug reproducer test must use real temporary storage (consistent with existing test patterns in `tests/content_display_integration_test.rs`)
- Bug reproducer must be marked `bug_reproducer(issue-036)` and follow the 5-section test documentation format

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note codestyle rules (2-space indent, no cargo fmt) and test organization standards.
2. **Write Test Matrix** — populate all four rows before opening any test file.
3. **Write failing test** — add `show_finds_session_in_topic_dir` to `tests/content_display_integration_test.rs` marked `bug_reproducer(issue-036)`. Setup: create temp storage, create project dir `{encoded_cwd}--commit`, write one `.jsonl` session file there, do NOT create the base `{encoded_cwd}` project dir. Run `clg .show session_id::{id}` from that cwd (or with CLAUDE_STORAGE_ROOT set). Confirm the test fails with "Session not found" before the fix.
4. **Implement fix** — in `show_session_in_cwd_impl`:
   - Remove `storage.load_project_for_cwd()` call
   - Get `cwd` via `std::env::current_dir()`
   - Encode to `eb` via `encode_path(&cwd)`
   - Call `storage.list_projects()` to get all projects
   - For each project where `dir_name == eb || dir_name.starts_with(&format!("{eb}--"))`, call `format_session_output(&project, session_id, verbosity, show_entries, metadata_only)`; return first `Ok` result
   - If no project matches or none contains the session, return `Err(ErrorData::new(ErrorCode::InternalError, format!("Session not found: {session_id}")))`
   - Add `Fix(issue-036)` comment with Root cause, Pitfall
5. **Add scope to YAML** — add `scope` parameter entry to the `.show` command block in `unilang.commands.yaml` (kind: String, optional, default: `local`).
6. **Green state** — `w3 .test level::3` must pass with zero failures and zero warnings.
7. **Walk Validation Checklist** — every answer must be YES.
8. **Update task status** — ✅ in `task/readme.md`, recalculate advisability to 0 (Priority=0), re-sort, move file to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `.show session_id::X` where X lives in `{encoded_cwd}--commit` topic dir only (no base project dir) | `show_session_in_cwd_impl` scope::local scan | Session found, content displayed, exit 0 |
| T02 | `.show session_id::X` where X lives in base project dir `{encoded_cwd}` (no topic dirs) | `show_session_in_cwd_impl` scope::local scan | Session found, content displayed, exit 0 (no regression) |
| T03 | `.show session_id::X` where X exists in no local project dirs | `show_session_in_cwd_impl` scope::local scan | "Session not found: X" error, exit nonzero |
| T04 | `.show session_id::X` where X lives in `{encoded_cwd}--default-topic` topic dir | `show_session_in_cwd_impl` scope::local scan | Session found, content displayed, exit 0 |

## Acceptance Criteria

- `clg .show session_id::9e9bc9ea` succeeds from `~/pro/lib/wip_core/willbe/dev/module/dream` when the session is in the `--commit` topic project dir
- T01 test passes: session in `--commit` dir found by `.show session_id::`
- T02 test passes: session in base dir still found (no regression in existing test cases)
- T03 test passes: non-existent session still produces "Session not found" error
- `w3 .test level::3` passes with zero failures

## Validation

### Checklist

Desired answer for every question is YES.

**Bug fix behavior**
- [x] C1 — Does `.show session_id::X` find a session stored in a `--commit` topic project dir?
- [x] C2 — Does `.show session_id::X` still find a session stored in the base project dir (no regression)?
- [x] C3 — Does `.show session_id::nonexistent` still return "Session not found" error?
- [x] C4 — Does `.show session_id::X` find a session stored in a `--default-topic` topic project dir?

**Implementation**
- [x] C5 — Is `load_project_for_cwd()` replaced (not merely supplemented) in `show_session_in_cwd_impl`?
- [x] C6 — Does the predicate use `starts_with(&format!("{eb}--"))` (double-hyphen prefix) not `starts_with(&format!("{eb}-"))` (single-hyphen)?
- [x] C7 — Is `Fix(issue-036)` source comment present with Root cause and Pitfall fields?

**YAML**
- [x] C8 — Is `scope` parameter registered in `.show` command block in `unilang.commands.yaml`?

**Tests**
- [x] C9 — Does the new test have the `bug_reproducer(issue-036)` marker?
- [x] C10 — Does the new test have 5-section documentation (Root Cause, Why Not Caught, Fix Applied, Prevention, Pitfall)?

**Out of Scope confirmation**
- [x] C11 — Are Case 1, Case 3, and Case 4 of `show_routine` unchanged?
- [x] C12 — Is `decode_project_display` unchanged (Task 025 territory)?

### Measurements

- [x] M1 — test count: `cargo nextest run --all-features 2>&1 | grep "tests run"` → count increases by ≥ 1 compared to before this task

### Invariants

- [x] I1 — test suite: `w3 .test level::3` → 0 failures
- [x] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --all-features` → 0 warnings

### Anti-faking checks

**AF1 — load_project_for_cwd removed from show_session_in_cwd_impl**
Check: `grep -n "load_project_for_cwd" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs`
Expected: zero matches inside `show_session_in_cwd_impl`. Why: confirms the exact-match lookup is gone and not just bypassed.

**AF2 — topic dir predicate uses double-hyphen**
Check: `grep -n 'starts_with.*eb.*--' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs | grep show`
Expected: ≥ 1 match. Why: single-hyphen `{eb}-` would incorrectly match child directories; double-hyphen `{eb}--` is the topic dir separator.

**AF3 — bug reproducer test exercises the topic-dir case**
Check: `grep -c "commit\|topic" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/content_display_integration_test.rs`
Expected: ≥ 2. Why: confirms the test actually creates a topic-dir scenario, not just a base-dir scenario.

## Outcomes

Replaced `storage.load_project_for_cwd()` (exact-match only) in `show_session_in_cwd_impl` with a `list_projects()` scan filtered by the scope::local predicate: `dir_name == eb || dir_name.starts_with(&format!("{eb}--"))`. The double-hyphen prevents false matches on sibling directories. Registered `scope` parameter (default `local`) in the `.show` YAML block. Four regression tests added (T05–T08 in `tests/content_display_integration_test.rs`) with full 5-section bug_reproducer documentation. `w3 .test level::3` passes with 317 tests, 0 failures, 0 warnings.

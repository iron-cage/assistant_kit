# Remove `.projects` summary mode — make list mode the only output mode

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** ✅ (Completed)
- **Validated By:** claude-sonnet-4-6
- **Validation Date:** 2026-04-22

## Goal

Remove the `is_default` summary mode branch from `projects_routine` and delete `render_active_project_summary`, so that bare `clg .projects` produces the same list output as any explicit invocation — eliminating the broken single-project selection (root causes H1+H2+H4 from investigation) and the hidden N-1 projects problem (root causes H1+H2+H3+H4 from second investigation). (Motivated: bare `clg .projects` shows a single subdirectory project as "Active project" due to `scope::under` + global-most-recent selection — users see the wrong project and miss all others; Observable: `clg .projects` with no args outputs `Found N projects:` list format, never a single-project summary block; Scoped: `src/cli/mod.rs` § `projects_routine` and `render_active_project_summary` only — no other commands affected; Testable: `cargo nextest run --test projects_command_test` passes with IT-1 rewritten to assert list-mode output and summary-mode helpers removed.)

The `is_default` gate (mod.rs:2453-2458) detects bare invocation and routes to `render_active_project_summary` (mod.rs:2388-2419), which takes `summaries.into_iter().next()?` — unconditionally discarding all but the globally-most-recently-active project. With `scope::under` as discovery default, subdirectory projects like `wplan_daemon/docs/-default_topic` can silently win the recency ranking and appear as "Active project" while the cwd project is hidden.

The fix removes this entire path. After task-018 (`scope::around` implementation), bare `clg .projects` uses `scope::around` and renders the full list — ancestors, current directory, and descendants — giving the user the neighborhood view they actually need.

**Dependency**: This task requires task-018 (scope::around implementation) to be complete first. The list mode default uses `scope::around`, which task-018 adds to `project_matches` and sets as `unwrap_or("around")`.

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs`
  § `projects_routine` lines ~2453-2458 — remove `is_default` variable
  § `projects_routine` lines ~2659-2664 — remove `if is_default { return ... }` summary branch
  § `render_active_project_summary` function lines ~2388-2419 — delete entirely
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/projects_command_test.rs`
  § `it1_default_shows_active_project_summary` — rewrite: assert list output (`Found N projects:`) instead of summary block
  § `it47_verbosity_1_alone_stays_in_summary_mode` — remove: mode concept no longer exists
  § `it34_explicit_scope_keeps_list_mode` — remove or rewrite: mode-boundary rationale obsolete
  § `it35_explicit_limit_keeps_list_mode` — remove or rewrite: mode-boundary rationale obsolete
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/readme.md` — update test count

## Out of Scope

- `scope::around` implementation — covered in task-018 (prerequisite)
- Documentation updates — already completed by doc_tsk in this session
- Other commands (`.list`, `.show`, etc.)
- Adding a new `mode::summary` parameter for opt-in summary behavior

## Requirements

- All work must strictly adhere to all applicable rulebooks
  (discover via `kbase .rulebooks`)
- No mocking — all tests use real `clg` binary via `common::clg_cmd()`
- TDD: rewrite failing test first, confirm it fails, then remove summary mode code
- `render_active_project_summary` must be fully deleted — not commented out or guarded
- The `is_default` variable must not remain in the source — no dead code

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note `code_style.rulebook.md` for deletion style and `test_organization.rulebook.md` for IT renaming.
2. **Confirm task-018 complete** — verify `grep -c '"around"' src/cli/mod.rs` ≥ 3 and `grep -c 'unwrap_or.*"around"' src/cli/mod.rs` ≥ 1. If task-018 is not done, implement it first.
3. **Read source** — Read `src/cli/mod.rs` lines 2386–2470 and lines 2650–2670 to understand full extent of the summary mode code path.
4. **Read tests** — Read `tests/projects_command_test.rs` functions `it1_`, `it47_`, `it34_`, `it35_` to understand what they currently assert.
5. **Write Test Matrix** — populate every row before touching any test code.
6. **Rewrite IT-1** — Change `it1_default_shows_active_project_summary` to assert `s.contains("Found")` and `!s.contains("Active project")` for bare invocation. Confirm test FAILS (still shows summary mode output).
7. **Remove summary mode code** — Delete:
   a. `is_default` variable block (lines ~2453-2458)
   b. `if is_default { ... }` branch (lines ~2659-2664)
   c. `render_active_project_summary` function (lines ~2388-2419)
8. **Validate targeted** — `cargo nextest run --test projects_command_test it1` → must pass.
9. **Clean up mode-boundary tests** — Remove `it47_`, `it34_`, `it35_` or rewrite them to test meaningful assertions about explicit param behavior (not mode switching).
10. **Validate full suite** — `w3 .test level::3` — zero failures, zero warnings, clippy clean.
11. **Update tests/readme.md** — update test count to reflect removed/rewritten tests.
12. **Walk Validation Checklist** — check every item. Every answer must be YES.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `clg .projects` (bare, from project with sessions) | Default — no args | `Found N projects:` header present; `Active project` absent |
| T02 | `clg .projects` (bare, from project with subdirectory project more recently active) | Default — no args | Cwd project AND subdirectory project both listed; no single-winner selection |
| T03 | `clg .projects scope::local` | Explicit scope | Same output format as bare (list); only cwd project shown |
| T04 | `clg .projects verbosity::1` | Explicit verbosity only | `Found N projects:` format (no longer triggers summary mode) |

## Acceptance Criteria

- `grep -c 'is_default' src/cli/mod.rs` returns `0`
- `grep -c 'render_active_project_summary' src/cli/mod.rs` returns `0`
- `grep -c 'Active project' tests/projects_command_test.rs` contains no positive assertions (only negative absence checks if any)
- `it1_default_shows_active_project_summary` renamed/rewritten to assert `s.contains("Found")`
- `it47_verbosity_1_alone_stays_in_summary_mode` no longer exists in `tests/projects_command_test.rs`
- `cargo nextest run --test projects_command_test` exits 0
- `w3 .test level::3` exits 0

## Validation

### Checklist

Desired answer for every question is YES.

**Summary mode removal**
- [ ] Is `is_default` variable absent from `src/cli/mod.rs`?
- [ ] Is `render_active_project_summary` function absent from `src/cli/mod.rs`?
- [ ] Is the `if is_default { return ... }` branch absent from `projects_routine`?

**Test correctness**
- [ ] Does IT-1 (renamed/rewritten) assert `s.contains("Found")` for bare invocation?
- [ ] Does IT-1 assert `!s.contains("Active project")` (summary block absent)?
- [ ] Is `it47_verbosity_1_alone_stays_in_summary_mode` absent from the test file?
- [ ] Do all remaining `it*_` tests pass?

**Behavior**
- [ ] Does `clg .projects` (bare) output `Found N projects:` format?
- [ ] Does `clg .projects verbosity::1` (explicit param) also output `Found N projects:` format?

**Out of Scope confirmation**
- [ ] Is `src/cli/mod.rs` outside `projects_routine` and `render_active_project_summary` unchanged?
- [ ] Are all commands other than `.projects` unaffected?

### Measurements

- [ ] M1 — `is_default` absent: `grep -c 'is_default' src/cli/mod.rs` → `0` (was: ≥6)
- [ ] M2 — summary function absent: `grep -c 'render_active_project_summary' src/cli/mod.rs` → `0` (was: 2)
- [ ] M3 — IT-1 passes: `cargo nextest run --test projects_command_test it1 2>&1 | tail -1` → `1 passed` (was: passes with old assertions)
- [ ] M4 — full suite: `w3 .test level::3 2>&1 | grep "^Summary:"` → all crates passed, 0 failures

### Invariants

- [ ] I1 — test suite: `w3 .test level::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check` → 0 warnings

### Anti-faking checks

- [ ] AF1 — `is_default` truly gone: `grep -n 'is_default' src/cli/mod.rs` → empty. Why: confirms the gate was deleted, not just unreachable.
- [ ] AF2 — `render_active_project_summary` truly gone: `grep -rn 'render_active_project_summary' src/` → empty. Why: confirms function is fully removed, not just uncalled.
- [ ] AF3 — IT-1 uses positive assertion: `grep -A 10 "fn it1_" tests/projects_command_test.rs | grep -c "contains.*Found"` → ≥1. Why: confirms the test actually validates list-mode output, not just absence of summary text.

## Outcomes

- Deleted `render_active_project_summary` function (~36 lines) from `src/cli/mod.rs`
- Deleted `is_default` variable block and the `if is_default { return ... }` branch from `projects_routine`
- Deleted `TRUNCATE_THRESHOLD`, `TRUNCATE_PREVIEW` constants and `last_text_entry`, `truncate_message` helper functions (all now dead without summary mode)
- Pruned `ProjectSummary` struct to `{ display_path, last_mtime }` — removed `session_count`, `last_session_id`, `last_session_entries`, `last_message` fields and their computation in `aggregate_projects`
- Updated `aggregate_projects` doc comment to remove stale references to session count and last-text-entry aggregation
- Rewrote IT-1 (`it1_default_shows_list_output`): asserts `Found` present, `Active project` absent for bare invocation
- Removed summary-mode tests: IT-30, IT-31, IT-32, IT-34, IT-35, IT-47-summary (verbosity gate), IT-50, IT-51
- Rewrote IT-33 to assert `Found 0 projects:` (list-mode zero-result header replaces summary sentinel)
- Updated `tests/readme.md`: count 287 → 289, removed "mode boundaries" description from `projects_command_test.rs` row, updated `projects_output_format_test.rs` row to reference IT-52..53
- Full Level 3 suite: 289 tests, exit 0, clippy clean

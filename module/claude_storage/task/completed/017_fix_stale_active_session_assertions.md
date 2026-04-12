# Fix stale "Active session" mode-boundary assertions in projects_command_test.rs

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** ✅ (Completed)
- **Validated By:** claude-sonnet-4-6 (independent subprocess validation)
- **Validation Date:** 2026-04-12

## Goal

Replace obsolete `!s.contains("Active session")` assertions in mode-boundary tests with the current summary-mode marker `!s.contains("Active project")`, and remove redundant occurrences in summary-mode tests, so that IT-34 and IT-35 catch regressions to the task-016 project-centric format rather than only to the pre-task-007 format. (Motivated: task-016 renamed the summary-mode header from "Active session" to "Active project"; the list-mode tests IT-34/IT-35 that guard against summary-mode leakage still check for the old marker and would pass even if the code accidentally output "Active project" in list mode — a real post-016 regression path; Observable: `grep -c '"Active session"' tests/projects_command_test.rs` returns 0 after the fix; Scoped: `tests/projects_command_test.rs` § `it1_`, `it47_`, `it34_`, `it35_` — assertion string updates and doc comment corrections only; Testable: `cargo nextest run --test projects_command_test it1 it47 it34 it35` passes with all assertions updated.)

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/projects_command_test.rs`
  § `it1_default_shows_active_project_summary` — remove redundant `!s.contains("Active session")` (line 1510); fix doc comment line 1484
  § `it47_verbosity_1_alone_stays_in_summary_mode` — remove redundant `!s.contains("Active session")` (line 1572); fix doc comment
  § `it34_explicit_scope_keeps_list_mode` — change `!s.contains("Active session")` → `!s.contains("Active project")` (line 1807); fix doc comment line 1783
  § `it35_explicit_limit_keeps_list_mode` — change `!s.contains("Active session")` → `!s.contains("Active project")` (line 1846); fix doc comment line 1822

## Out of Scope

- Documentation updates (already completed by doc_tsk)
- `tests/projects_output_format_test.rs` — its "Active session" mentions (lines 25, 446, 448) are historical context in doc comments explaining what task-016 fixed; these must NOT be changed (they describe the bug that was fixed)
- Any changes to assertion logic beyond the four named functions
- Source code changes in `src/`

## Description

Task-016 redesigned the `.projects` summary-mode output, replacing the header `Active session` (introduced by task-007) with `Active project`. The mode-boundary tests IT-34 and IT-35 guard against summary-mode leakage when explicit parameters are given — they assert that list mode is active by checking `s.contains("Found")` (positive) and `!s.contains("Active session")` (negative).

The negative check `!s.contains("Active session")` now tests for text the implementation no longer produces under any code path. This means if task-016's implementation were to accidentally activate summary mode when an explicit scope is given, outputting `Active project` instead of `Found N projects:`, IT-34 and IT-35 would still PASS the negative assertion (because "Active session" ≠ "Active project"). The positive check `s.contains("Found")` would catch this, but the negative check documents the wrong invariant and misleads future readers about what summary mode looks like.

For the summary-mode tests IT-1 and IT-47, `!s.contains("Active session")` is entirely redundant: if `s.contains("Active project")` passes (already asserted), "Active session" cannot appear. Remove it.

The fix is purely assertion-level — no production code changes required.

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- No mocking; all tests use real `clg` binary via `common::clg_cmd()`
- Keep the 5-section doc-comment format for bug-reproducer tests (IT-47 already has it; do not remove sections)
- Only update assertions and doc comments in the four named functions — no logic changes

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note `test_organization.rulebook.md` doc-comment format requirements.
2. **Read test functions** — Read `tests/projects_command_test.rs` lines 1480–1520 (it1), 1543–1580 (it47), 1760–1810 (it34), 1810–1860 (it35) to understand current assertions.
3. **Fix it1** — Remove `assert!( !s.contains( "Active session" ), ... )` (line 1510). Update doc comment line 1484 to remove `(not "Active session")` clause.
4. **Fix it47** — Remove `assert!( !s.contains( "Active session" ), ... )` (line 1572). Update the corresponding doc comment.
5. **Fix it34** — Change `assert!( !s.contains( "Active session" ), ... )` to `assert!( !s.contains( "Active project" ), "explicit scope:: must not show 'Active project' summary; got:\n{s}" )`. Update doc comment to say "does NOT contain 'Active project'".
6. **Fix it35** — Same change as it34: `"Active session"` → `"Active project"`. Update doc comment.
7. **Validate** — Run `cargo nextest run --test projects_command_test it1 it47 it34 it35`. All 4 tests must pass.
8. **Validate full suite** — Run `w3 .test level::3`. All tests must pass.
9. **Walk Validation Checklist** — check every item. Every answer must be YES.

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `clg .projects` (bare) | project with session at cwd | Summary mode: `Active project` present, no `Active session` assertion (removed), no `Found` |
| `clg .projects verbosity::1` | project with session at cwd | Summary mode: `Active project` present, no `Active session` assertion (removed) |
| `clg .projects scope::local` | project at explicit path | List mode: `Found` present, `Active project` NOT present (new assertion) |
| `clg .projects limit::5` | project with session | List mode: `Found` present, `Active project` NOT present (new assertion) |

## Acceptance Criteria

- `grep -c '"Active session"' tests/projects_command_test.rs` returns `0` (all occurrences removed)
- `it34_explicit_scope_keeps_list_mode` asserts `!s.contains("Active project")` (not "Active session")
- `it35_explicit_limit_keeps_list_mode` asserts `!s.contains("Active project")` (not "Active session")
- `it1_default_shows_active_project_summary` no longer asserts `!s.contains("Active session")`
- `it47_verbosity_1_alone_stays_in_summary_mode` no longer asserts `!s.contains("Active session")`
- `cargo nextest run --test projects_command_test it1 it47 it34 it35` exits 0 (4 tests pass)
- `w3 .test level::3` exits 0 (full suite clean)

## Validation

### Checklist

Desired answer for every question is YES.

**Core assertion fixes**
- [x] Does `grep -c '"Active session"' tests/projects_command_test.rs` return `0`?
- [x] Does IT-34 assert `!s.contains("Active project")`?
- [x] Does IT-35 assert `!s.contains("Active project")`?
- [x] Is the `!s.contains("Active session")` assertion absent from IT-1?
- [x] Is the `!s.contains("Active session")` assertion absent from IT-47?

**Test correctness**
- [x] Does IT-34 still assert `s.contains("Found")` (positive list-mode check unchanged)?
- [x] Does IT-35 still assert `s.contains("Found")` (positive list-mode check unchanged)?
- [x] Does IT-1 still assert `s.contains("Active project")` (summary mode positive check unchanged)?
- [x] Do all four test functions (it1, it47, it34, it35) pass?

**Out of Scope confirmation**
- [x] Are "Active session" mentions in `projects_output_format_test.rs` (historical context) unchanged?
- [x] Is `src/` unchanged (no production code modifications)?

### Measurements

**M1 — "Active session" assertion count reaches 0**
Command: `grep -c '"Active session"' tests/projects_command_test.rs`
Before: `7` (assertions: lines 1510, 1572, 1807, 1846; doc comments: lines 1484, 1783, 1822). Expected: `0`. Deviation: non-zero means fix was incomplete.

**M2 — IT-34 and IT-35 now assert "Active project" absence**
Command: `grep -A2 "fn it34_\|fn it35_" tests/projects_command_test.rs | grep "Active project" | wc -l`
Before: `0`. Expected: `2`. Deviation: fewer than 2 means one of the fixes was missed.

**M3 — Full test suite passes**
Command: `w3 .test level::3 2>&1 | grep "^Summary:"`
Expected: `Summary: 13/13 crates passed, 0 failed`. Deviation: any failure.

### Anti-faking checks

**AF1 — it34 specifically asserts list mode via Active project absence**
Check: `grep -A 5 "fn it34_" tests/projects_command_test.rs | grep 'Active project'`
Expected: non-empty (assertion present). Why: confirms the fix was applied to it34 specifically and not accidentally skipped.

**AF2 — it1 no longer has the redundant Active session check**
Check: `grep -A 10 "fn it1_default" tests/projects_command_test.rs | grep '"Active session"' | wc -l`
Expected: `0`. Why: confirms the redundant assertion was removed from it1.

## Outcomes

- **Completed:** 2026-04-12
- **M1:** `grep -c '"Active session"' tests/projects_command_test.rs` → `0` ✅ (was 7)
- **M2:** IT-34 and IT-35 now assert `!s.contains("Active project")` ✅
- **M3:** `w3 .test level::3` → 281 tests + 3 doc tests passed, 0 failures, 0 warnings, clippy clean ✅
- **Changes:** 7 string changes in `tests/projects_command_test.rs` — 4 assertions and 3 doc comments
- **OOF file:** `tests/projects_output_format_test.rs` unchanged (5 historical "Active session" references preserved) ✅
- **src/:** Zero production code changes ✅

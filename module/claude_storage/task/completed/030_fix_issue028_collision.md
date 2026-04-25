# Fix issue-028 ID Collision — Rename cwd-default Markers to issue-037

## Execution State

- **Executor Type:** any
- **Actor:** claude-sonnet-4-6
- **Claimed At:** 2026-04-25
- **Status:** ✅ (Completed)
- **Validated By:** claude-sonnet-4-6
- **Validation Date:** 2026-04-25

## Goal

Rename the `bug_reproducer(issue-028)` annotations in `tests/session_path_command_test.rs` to `bug_reproducer(issue-037)` and add a `Fix(issue-037)` source comment to `resolve_session_dir` in `src/cli/mod.rs`, resolving the ID collision where issue-028 currently refers to two unrelated bugs (Motivated: `Fix(issue-028)` at lines 1201 and 1377 of `src/cli/mod.rs` already identifies the plural "entry/entries" display bug; `bug_reproducer(issue-028)` added during task 028 misappropriates that ID for a different cwd-default fix; Observable: `grep -c 'bug_reproducer(issue-028)' tests/session_path_command_test.rs` → 0; Scoped: `tests/session_path_command_test.rs` marker rename + `src/cli/mod.rs` Fix comment addition; Testable: `w3 .test level::3` passes after the rename).

The Ubiquitous Language Enforcement principle (principles.rulebook.md § Content Integrity : Ubiquitous Language Enforcement) requires one canonical term per domain concept. An issue ID is such a term — it must uniquely identify exactly one bug. Issue-037 is the next available number after issue-036 (both issue-035 and issue-036 are already used in `src/cli/mod.rs`).

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/session_path_command_test.rs`
  - Replace `bug_reproducer(issue-028)` → `bug_reproducer(issue-037)` in test functions `it_session_dir_cwd_default` and `it_session_ensure_cwd_default` (2 occurrences)
  - Update any doc comment lines that reference `issue-028` in those test blocks to `issue-037`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs`
  - Add a `// Fix(issue-037):` comment (3-field format) to the `resolve_session_dir` function

## Out of Scope

- `Fix(issue-028)` comments at lines ~1201 and ~1377 of `src/cli/mod.rs` — those belong to the plural "entry/entries" bug and must not change
- Behavioral changes — cwd-default logic was implemented in task 028 and is correct

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- The replacement is exhaustive: **no** `bug_reproducer(issue-028)` annotation must remain in `session_path_command_test.rs`
- The Fix comment must use the 3-field format from code_style.rulebook.md: `Fix(issue-037)`, `Root cause`, `Pitfall`
- Depends on task 029 (same test file): execute task 029 first to avoid merge conflicts

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note codestyle constraints.
2. **Confirm task 029 is complete** — `grep -c 'pub fn path_routine\|pub fn exists_routine' src/cli/mod.rs` → 0. If non-zero, complete task 029 first.
3. **Read source** — Read `tests/session_path_command_test.rs` to locate both `bug_reproducer(issue-028)` occurrences and their surrounding doc comments.
4. **Read source** — Read `src/cli/mod.rs` around `resolve_session_dir` to find the exact insertion point for the Fix comment.
5. **Update tests** — Replace `bug_reproducer(issue-028)` → `bug_reproducer(issue-037)` in both test blocks; update any `issue-028` references in the surrounding `//` doc comment lines.
6. **Add Fix comment** — Insert the 3-field `// Fix(issue-037)` comment into `resolve_session_dir` above its signature or in the function body preamble (before the first `let` statement), using the format:
   ```
   // Fix(issue-037): resolve_session_dir defaults to cwd when path:: absent
   // Root cause: old resolve_required_session_dir required path:: and rejected bare invocations
   // Pitfall: resolve_cmd_path returns cwd — callers in tests must not assume absolute input is required
   ```
7. **Verify unchanged** — `grep -c 'Fix(issue-028)' src/cli/mod.rs` → 2 (unchanged).
8. **Validate** — `w3 .test level::3`. All tests must pass, zero warnings.
9. **Walk Validation Checklist** — every item must answer YES.
10. **Update task status** — set ✅ in `task/readme.md`, recalculate Advisability to 0, re-sort, move file to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Test `it_session_dir_cwd_default` passes | new marker `issue-037` | Exit 0; test passes |
| T02 | Test `it_session_ensure_cwd_default` passes | new marker `issue-037` | Exit 0; test passes |
| T03 | `grep bug_reproducer(issue-028) tests/` | old marker absent | → 0 matches |
| T04 | `grep Fix(issue-037) src/cli/mod.rs` | new source comment | → 1 match |
| T05 | `grep Fix(issue-028) src/cli/mod.rs` | plural-bug markers | → 2 matches (unchanged) |

## Acceptance Criteria

- `bug_reproducer(issue-028)` is absent from `tests/session_path_command_test.rs`
- `bug_reproducer(issue-037)` appears exactly twice in `tests/session_path_command_test.rs`
- `Fix(issue-037)` appears exactly once in `src/cli/mod.rs`, inside `resolve_session_dir`
- `Fix(issue-028)` still appears exactly twice in `src/cli/mod.rs` (plural-bug markers unchanged)
- `w3 .test level::3` passes with 0 failures and 0 warnings

## Validation

### Checklist

Desired answer for every question is YES.

- [ ] C1 — Is `bug_reproducer(issue-028)` absent from `tests/session_path_command_test.rs`?
- [ ] C2 — Does `tests/session_path_command_test.rs` contain exactly 2 `bug_reproducer(issue-037)` markers?
- [ ] C3 — Does `src/cli/mod.rs` contain exactly 1 `Fix(issue-037)` comment?
- [ ] C4 — Does `src/cli/mod.rs` still contain exactly 2 `Fix(issue-028)` comments?
- [ ] C5 — Does `w3 .test level::3` pass with 0 failures and 0 warnings?

### Measurements

- [ ] M1 — old marker absent: `grep -c 'bug_reproducer(issue-028)' tests/session_path_command_test.rs` → `0`
- [ ] M2 — new marker present: `grep -c 'bug_reproducer(issue-037)' tests/session_path_command_test.rs` → `2`
- [ ] M3 — Fix comment added: `grep -c 'Fix(issue-037)' src/cli/mod.rs` → `1`
- [ ] M4 — plural-bug unchanged: `grep -c 'Fix(issue-028)' src/cli/mod.rs` → `2`

### Invariants

- [ ] I1 — test suite: `w3 .test level::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --all-features` → 0 warnings

### Anti-faking checks

- [ ] AF1 — no old marker anywhere: `grep -rn 'bug_reproducer(issue-028)' tests/` → 0 matches
- [ ] AF2 — new Fix comment inside resolve_session_dir: `grep -n 'Fix(issue-037)' src/cli/mod.rs` → line number falls within `resolve_session_dir` function body
- [ ] AF3 — plural-bug markers intact: `grep -c 'Fix(issue-028)' src/cli/mod.rs` → `2`

## Outcomes

- Replaced all 5 occurrences of `issue-028` in `tests/session_path_command_test.rs` with `issue-037` (coverage section + 2 `bug_reproducer` markers + 2 doc comment lines)
- Added `Fix(issue-037)` 3-field comment to `resolve_session_dir` in `src/cli/mod.rs`
- Confirmed `Fix(issue-028)` at lines 1201 and 1377 of `src/cli/mod.rs` remain unchanged (plural-bug markers)
- Note: issue-035 was already used at line 2126 of `src/cli/mod.rs` (decode_project_display topic-loop fix from task 025); issue-037 was chosen as the next free number after scanning all issue IDs in use
- 318/319 tests pass; B17 live-storage failure is pre-existing

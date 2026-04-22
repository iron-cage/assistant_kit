# Update `.projects` output to use conversation terminology consistently

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** ✅ (Complete)

## Goal

Update `.projects` CLI output and documentation so that "conversation" is the consistent user-facing term for all verbosity levels (Motivated: the current output says "(N sessions)" for agent-free projects and "(N conversations, M agents)" only when agents are present — this is inconsistent; sessions are implementation details that should not appear in user-facing output; Observable: `clg .projects` output headers always say "conversations", session UUIDs are labelled as conversation identifiers, and `commands.md` verbosity matrix uses "conversation" uniformly; Scoped: output text changes in `src/cli/mod.rs`, documentation updates in `docs/cli/commands.md`; no structural CLI behavior changes; Testable: `w3 .test level::3` passes and `grep "(N sessions)" tests/` returns zero matches).

**Prerequisite:** Task 021 (`Conversation` type introduced).

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs`
  - Change project header display: replace `"(N sessions)"` (no-agent case) with `"(N conversations)"`
  - Remove the conditional that switches between "sessions" and "conversations" noun based on agent presence — always use "conversations"
  - Update the `Found N projects:` format description in code comments
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/commands.md`
  - Update verbosity matrix: all rows use "conversations" in project header column
  - Update any "(N sessions)" examples in the verbosity output format section to "(N conversations)"
  - Update notes where "sessions" is used to describe what `.projects` displays
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/projects_command_test.rs`
  - Update any test assertions that check for "(N sessions)" output to check for "(N conversations)"
  - Add test `it_header_uses_conversations_not_sessions` confirming the header always uses "conversations"

## Out of Scope

- Hiding session UUIDs from output (currently shown as conversation identifiers at v1+; task for future)
- Implementing conversation chain grouping (→ task 021 already done; this task uses the 1:1 mapping)
- Adding new parameters to `.projects`
- Changes to `.list` or `.count` commands (→ task 023)

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Output change must be validated against `tests/projects_command_test.rs` and `tests/projects_output_format_test.rs` before completion
-   `commands.md` must be updated in the same PR as the code change (no stale docs)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note CLI doc update requirements.
2. **Read source** — Read `src/cli/mod.rs` `render_project_families` function and the project header formatting code; identify all places where the word "session" or "sessions" is used in user-visible output strings.
3. **Read docs** — Read `docs/cli/commands.md` §`.projects` verbosity output format section; identify all "session" occurrences that should become "conversation".
4. **Read taxonomy doc** — Read `docs/claude_code/007_concept_taxonomy.md` §Why Sessions Are a Hidden Detail for the authoritative rationale.
5. **Write Test Matrix** — populate matrix before modifying any file.
6. **Write failing tests** — add `it_header_uses_conversations_not_sessions` to `tests/projects_command_test.rs`; run `w3 .test level::3` and confirm new test fails (assertion checks for output not yet changed).
7. **Implement** — change all "session"/"sessions" noun strings in user-facing output in `src/cli/mod.rs` to "conversation"/"conversations". Apply conditional removal (always "conversations").
8. **Update docs** — Update `commands.md` verbosity matrix and examples to use "conversations" uniformly.
9. **Green state** — `w3 .test level::3` passes with zero failures and zero warnings.
10. **Walk Validation Checklist** — every answer must be YES.
11. **Update task status** — ✅ in `task/readme.md`, move to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `clg .projects` on project with sessions, no agents | default scope::around | Header shows `(N conversations)` not `(N sessions)` |
| T02 | `clg .projects` on project with sessions and agents | default scope::around | Header shows `(N conversations, M agents)` — same as before but no regression |
| T03 | `clg .projects` on empty storage | any scope | `Found 0 projects:` (no "sessions" noun) |

## Acceptance Criteria

- `grep "(N sessions)" src/cli/mod.rs` returns zero matches (format string removed)
- Project header for agent-free projects shows `(N conversations)` in test output
- `commands.md` verbosity matrix column shows "conversations" for all verbosity levels
- All 290+ tests pass with zero warnings

## Validation

### Checklist

Desired answer for every question is YES.

**Code output**
- [ ] C1 — Does `grep '"sessions"' src/cli/mod.rs` return zero matches in user-visible format strings?
- [ ] C2 — Does a project with sessions but no agents show `(N conversations)` in its header?
- [ ] C3 — Does a project with sessions and agents show `(N conversations, M agents)` in its header?

**Documentation**
- [ ] C4 — Does `commands.md` verbosity matrix use "conversations" in all project header cells?
- [ ] C5 — Are all output examples in `commands.md` §`.projects` consistent with the new "conversations" noun?

**Out of Scope confirmation**
- [ ] C6 — Is `.list` command output unchanged?
- [ ] C7 — Are `.count`, `.show`, `.search` commands unchanged?

### Measurements

- [ ] M1 — test count: `w3 .test level::3 2>&1 | grep "test result"` → `test result: ok. 291 passed` (was: 290 after task 021)

### Invariants

- [ ] I1 — test suite: `w3 .test level::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --all-features` → 0 warnings

### Anti-faking checks

**AF1 — Verify session noun removed from format strings**
Check: `grep -n '"sessions"' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs | grep -v "//"`
Expected: zero matches (excluding comments). Why: catches partial implementation where some display paths still use the old noun.

## Outcomes

[Added upon task completion]

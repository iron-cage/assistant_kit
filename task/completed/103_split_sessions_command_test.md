# Split `claude_storage/tests/sessions_command_test.rs` into focused test files

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** ✅ (Completed)

## Goal

Split the 2,484-line `claude_storage/tests/sessions_command_test.rs` into focused test files organized by test domain so that no single test file exceeds the 1,500-line limit (Motivated: the file violates the 1,500-line test file limit, making it difficult to navigate and increasing the cognitive load when adding new tests; Observable: multiple focused test files each under 1,000 lines replacing the monolith; Scoped: `tests/` directory only — no source changes; Testable: `w3 .test level::3` passes with zero regressions after the split).

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/sessions_command_test.rs` — split into:
  - `tests/sessions_scope_test.rs` — scope parameter semantics tests (local, relevant, under, global)
  - `tests/sessions_family_display_test.rs` — family display tests IT-36..IT-48 (conversation/agent header format, agent type propagation, orphan display)
  - `tests/sessions_edge_case_test.rs` — edge-case tests EC-1..EC-8
  - `tests/sessions_command_test.rs` — retain for any remaining integration tests not clearly categorized above; update module doc comment
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/readme.md` — update responsibility table to list new test files

## Out of Scope

- Changes to any source file in `src/`
- Changes to other test files
- Adding new test cases (pure reorganization)

## Description

`tests/sessions_command_test.rs` is 2,484 lines covering three distinct test domains: scope semantics (how `local`, `relevant`, `under`, `global` filter sessions), family display formatting (IT-36..IT-48 covering conversation/agent hierarchy rendering), and edge cases (EC-1..EC-8). These domains are independent and map naturally to separate files.

The split requires moving test functions and their shared fixtures/helpers into the appropriate file. Each new file must declare its own `mod common;` import (the shared `tests/common/` module). Doc comments must be updated to reflect the narrowed scope.

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- Zero behavior change — every test function moves verbatim; no test logic modified
- Each resulting file must be under 1,000 lines
- Module-level doc comments (`//!`) must accurately describe the narrowed scope of each file
- `tests/readme.md` must be updated to register all new test files

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note test organization requirements.
2. **Read source file** — Read `tests/sessions_command_test.rs` fully; identify test domains and shared fixtures.
3. **Write Test Matrix** — populate Test Matrix confirming split boundaries.
4. **Create scope test file** — Extract scope-semantics tests into `tests/sessions_scope_test.rs`; add doc comment.
5. **Create family display test file** — Extract IT-36..IT-48 into `tests/sessions_family_display_test.rs`; add doc comment.
6. **Create edge case test file** — Extract EC-1..EC-8 into `tests/sessions_edge_case_test.rs`; add doc comment.
7. **Update `sessions_command_test.rs`** — Keep remaining tests; update doc comment to reflect reduced scope.
8. **Update `tests/readme.md`** — Add rows for the 3 new test files.
9. **Green state** — `w3 .test level::3` must pass with zero failures and zero warnings.
10. **Verify sizes** — `wc -l module/claude_storage/tests/sessions*.rs` — every file must be under 1,000 lines.
11. **Update task status** — set ✅ in `task/readme.md`, move file to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `w3 .test level::3` after split | All 13 crates | 0 failures, 0 warnings |
| T02 | `wc -l tests/sessions*.rs` | All sessions test files | Every file ≤ 1000 lines |
| T03 | Test count before vs after | Both states | Same total test count |
| T04 | `cargo nextest run --test sessions_scope_test` | sessions_scope_test | All scope tests pass |
| T05 | `cargo nextest run --test sessions_family_display_test` | sessions_family_display_test | All IT-36..IT-48 pass |

## Acceptance Criteria

- Every `sessions_*.rs` test file is ≤ 1,000 lines
- Total test count is unchanged vs. pre-split baseline
- `w3 .test level::3` passes with 0 regressions
- `tests/readme.md` lists all new test files
- Each new file has an accurate `//!` module doc comment describing its scope

## Validation

### Checklist

Desired answer for every question is YES.

**File size**
- [ ] C1 — Is every `tests/sessions_*.rs` file ≤ 1,000 lines?
- [ ] C2 — Is no single sessions test file > 1,500 lines (hard limit)?

**Behavior preservation**
- [ ] C3 — Does `w3 .test level::3` pass with 0 failures?
- [ ] C4 — Is the total test count identical to the pre-split baseline?

**Documentation**
- [ ] C5 — Does each new test file have an accurate `//!` module doc comment?
- [ ] C6 — Does `tests/readme.md` list all 4 sessions test files?

**Out of Scope confirmation**
- [ ] C7 — Are no source files in `src/` modified?
- [ ] C8 — Are no other test files (non-sessions) modified?

### Measurements

- [ ] M1 — largest sessions test file: `wc -l module/claude_storage/tests/sessions*.rs | sort -rn | head -2` → max ≤ 1000 (was: 2484)
- [ ] M2 — test count: `cargo nextest list -p claude_storage 2>&1 | grep -c "sessions"` → same count before and after

### Anti-faking checks

- [ ] AF1 — file count: `ls module/claude_storage/tests/sessions*.rs | wc -l` → ≥ 3 (new files created)
- [ ] AF2 — no test removed: total count of `fn test_` or `#[test]` across all sessions files equals original count

## Outcomes

**Completed.** Target file had been renamed from `sessions_command_test.rs` → `projects_command_test.rs` in the `62a01f0` auto-commit (sessions→projects rename). `projects_command_test.rs` (1,241 lines) split into 3 focused files:

| File | Lines | Tests | Content |
|------|-------|-------|---------|
| `projects_command_test.rs` | 519 | 15 | Filters, validation, output formatting, IT-14..IT-16, IT-50 |
| `projects_edge_case_test.rs` | 271 | 9 | EC-1..EC-8 + EC-9 (root path rejection) |
| `projects_scope_test.rs` | 529 | 11 | Scope behavioral semantics, IT-9..IT-13 underscore, UUID, topic-dir |

**Deviation:** The planned target files used `sessions_` prefix (e.g. `sessions_scope_test.rs`) but since the source file was renamed to `projects_*`, the new files correctly use `projects_*` naming for consistency.

**Validation results:**
- 319/319 tests pass (same count as before split)
- `cargo clippy` → 0 warnings
- All files ≤ 1,000 lines ✓
- `tests/readme.md` updated (tree + responsibility table)

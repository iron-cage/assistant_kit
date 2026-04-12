# Split `claude_storage/src/cli/mod.rs` into per-command modules

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** 📥 (Backlog)

## Goal

Split the 3,109-line `claude_storage/src/cli/mod.rs` into focused per-command source files so that no single file exceeds the 1,500-line limit (Motivated: the file is more than 2× the hard limit, causing difficult navigation and merge conflicts when multiple commands are modified; Observable: 10 focused files replacing one monolith, each under 500 lines; Scoped: `src/cli/` directory only — no behavior change, no API surface change; Testable: `w3 .test level::3` passes with zero regressions after the split).

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs` — split into:
  - `src/cli/storage.rs` — `create_storage()` factory (lines 60-82)
  - `src/cli/status.rs` — `status_routine`, `resolve_path_parameter` (lines 83-326)
  - `src/cli/format.rs` — `format_entry_content`, `format_timestamp`, `truncate_if_needed` (lines 327-482) shared formatting utilities
  - `src/cli/list.rs` — `list_routine`, `parse_project_parameter` (lines 483-875)
  - `src/cli/show.rs` — `show_routine`, `show_project_routine`, all show-* impl helpers, `format_session_output`, `format_project_output` (lines 876-1367)
  - `src/cli/count.rs` — `count_routine` (lines 1368-1527)
  - `src/cli/search.rs` — `search_routine` (lines 1528-1760)
  - `src/cli/export.rs` — `export_routine` (lines 1761-1838)
  - `src/cli/session.rs` — `session_routine`, path-decode helpers, walk helpers, session-display utilities (lines 1839-2438)
  - `src/cli/sessions.rs` — `sessions_routine`, agent-meta helpers, family-display helpers (lines 2439-3109)
  - `src/cli/mod.rs` — module declarations + `pub use` re-exports only (< 30 lines)
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/readme.md` — update responsibility table to list new submodule files
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/` — new `readme.md` for `src/cli/` subdirectory (currently missing)

## Out of Scope

- Any behavior change to existing routines (pure structural refactor)
- Changes to tests (tests call the same public API, no modification needed)
- Changes to any other crate

## Description

`module/claude_storage/src/cli/mod.rs` is a 3,109-line monolith that contains all ten command routines plus shared formatting utilities. At 2× the 1,500-line limit it violates the file size invariant from `files_structure.rulebook.md` and makes navigation and focused editing difficult.

The natural split is along command boundaries: each public routine (status, list, show, count, search, export, session, sessions) becomes its own module file alongside a shared `format.rs` (formatting utilities used by multiple commands) and `storage.rs` (the storage factory). The surviving `mod.rs` becomes a thin module declaration file.

All public items are re-exported from `mod.rs` so the crate's `src/lib.rs` surface is unchanged. No callers inside or outside the crate are affected.

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- Zero behavior change — this is a pure structural refactor; every function moves verbatim
- Each resulting file must be under 500 lines (ideal target: 200-400 lines per command file)
- `mod.rs` must contain only `mod` declarations and `pub use` re-exports
- New `src/cli/readme.md` required (new directory-level registration)
- Update `src/readme.md` to register the new `cli/` subdirectory entry

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note file size limits and module organization rules.
2. **Read source file** — Read `src/cli/mod.rs` fully; identify every `use` import, type definition, and function boundary.
3. **Write Test Matrix** — populate Test Matrix confirming the split boundaries and expected output sizes.
4. **Create module files** — Extract each command group into its dedicated file using the line-range boundaries listed in In Scope. Copy verbatim; add any necessary `use` declarations for local types.
5. **Rewrite `mod.rs`** — Replace monolith with `mod` declarations and `pub use` re-exports.
6. **Create `src/cli/readme.md`** — Responsibility Table listing the 10 new files.
7. **Update `src/readme.md`** — Add `cli/` row.
8. **Green state** — `w3 .test level::3` must pass with zero failures and zero warnings.
9. **Verify sizes** — `wc -l module/claude_storage/src/cli/*.rs` — every file must be under 500 lines.
10. **Update task status** — set ✅ in `task/readme.md`, move file to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `w3 .test level::3` after split | All 13 crates | 0 failures, 0 warnings |
| T02 | `wc -l src/cli/*.rs` | All split files | Every file ≤ 500 lines |
| T03 | `wc -l src/cli/mod.rs` | Thin re-export file | ≤ 30 lines |
| T04 | `cargo check` in `claude_storage` | Default features | 0 errors, 0 warnings |

## Acceptance Criteria

- `src/cli/mod.rs` is ≤ 30 lines (declarations + re-exports only)
- Every new module file (`status.rs`, `list.rs`, `show.rs`, `count.rs`, `search.rs`, `export.rs`, `session.rs`, `sessions.rs`, `format.rs`, `storage.rs`) is ≤ 500 lines
- `w3 .test level::3` passes with 0 regressions vs. pre-refactor baseline
- `src/cli/readme.md` exists with Responsibility Table listing all 10 module files
- No existing test file is modified

## Validation

### Checklist

Desired answer for every question is YES.

**File size**
- [ ] C1 — Is `src/cli/mod.rs` ≤ 30 lines?
- [ ] C2 — Are all 10 new module files ≤ 500 lines each?
- [ ] C3 — Does no file in `src/cli/` exceed 500 lines?

**Behavior preservation**
- [ ] C4 — Does `w3 .test level::3` pass with 0 failures?
- [ ] C5 — Does `cargo clippy` report 0 warnings for `claude_storage`?

**Documentation**
- [ ] C6 — Does `src/cli/readme.md` exist with a Responsibility Table?
- [ ] C7 — Does `src/readme.md` list the `cli/` directory?

**Out of Scope confirmation**
- [ ] C8 — Are no test files in `tests/` modified?
- [ ] C9 — Are no other crates modified?

### Measurements

- [ ] M1 — cli/mod.rs size: `wc -l module/claude_storage/src/cli/mod.rs` → ≤ 30 (was: 3109)
- [ ] M2 — largest split file: `wc -l module/claude_storage/src/cli/*.rs | sort -rn | head -2` → max ≤ 500

### Anti-faking checks

- [ ] AF1 — module count: `ls module/claude_storage/src/cli/*.rs | wc -l` → 11 (10 command modules + mod.rs)
- [ ] AF2 — no dead code: `RUSTFLAGS="-D warnings" cargo check -p claude_storage --all-features` → 0 errors
- [ ] AF3 — no inlined logic in mod.rs: `grep -c "fn " module/claude_storage/src/cli/mod.rs` → 0

## Outcomes

[Added upon task completion.]

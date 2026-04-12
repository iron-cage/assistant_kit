# Rename `.sessions` command to `.projects` (mechanical rename)

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** ✅ (Complete)

## Goal

Rename `.sessions` → `.projects` across all code, YAML, test, and documentation layers, preserving all existing behavior, so that the command name aligns with the user-facing concept (project directories) rather than the implementation detail (session JSONL files). (Motivated: the `.sessions` name teaches users to think in "sessions" — JSONL files that accumulate per run — when users actually care about project directories; the name mismatch creates a conceptual mismatch between what the command shows and what users mentally model; Observable: `".projects"` in `src/cli_main.rs`, `fn projects_routine` in `src/cli/mod.rs`, `tests/projects_command_test.rs` exists, `tests/sessions_command_test.rs` absent, `docs/cli/commands.md` Command 7 heading reads `.projects`; Scoped: purely mechanical rename — zero behavioral changes, all existing tests continue to pass under the new name; Testable: `grep -c '".sessions"' src/cli_main.rs` → `0` AND `grep -c '".projects"' src/cli_main.rs` → `1` AND `cargo nextest run` → 0 failures.)

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs` § `sessions_routine` (line 2439) — rename function to `projects_routine`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli_main.rs` — change `".sessions" => cli::sessions_routine` → `".projects" => cli::projects_routine`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/unilang.commands.yaml` — rename `.sessions` command to `.projects`; update descriptions and examples
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/sessions_command_test.rs` → rename to `projects_command_test.rs`; update all `".sessions"` argument strings to `".projects"`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/sessions_output_format_test.rs` → rename to `projects_output_format_test.rs`; update all `.sessions` references
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/readme.md` — rename rows for both test files
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/commands.md` — rename Command heading, update syntax examples and all prose references from `.sessions` → `.projects`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/testing/command/sessions.md` → rename to `projects.md`; update internal `.sessions` references
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/testing/command/readme.md` — rename `sessions.md` row → `projects.md`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/dictionary.md` — update `Active Session` entry reference and `Scope` entry reference from `.sessions` → `.projects`

## Out of Scope

- Output behavior changes — behavior is IDENTICAL to current `.sessions`; Task 016 handles the redesign
- `.session.dir`, `.session.ensure`, `.session` (already removed by task 014), `.exists` — untouched
- Renaming the internal variable `is_default` inside the routine — it's an implementation detail, not user-facing

## Description

This is a purely mechanical rename. The `.sessions` command currently shows session families grouped by project, with a summary mode on bare invocation. After this task the command is invoked as `.projects` but behaves identically. Every test that previously passed with `".sessions"` will pass identically with `".projects"` — only the command argument string changes in test invocations.

The name change is important for the mental model. "Sessions" suggests the user is managing JSONL files; "projects" surfaces the directory (the thing users care about). This task establishes the correct name; Task 016 will then redesign the output to match the name.

The `is_default` guard logic (the bug fixed in task 012) must remain unchanged. The summary mode and list mode dispatch must continue to work identically.

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- Purely mechanical: zero behavioral changes — the `is_default` guard, scope logic, output format, all parameter semantics are unchanged
- No backup files — use `mv` or direct rename; git history preserves recovery
- Test files are renamed (not re-created): update file contents in place after rename

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note `code_design.rulebook.md` rename guidance.
2. **Read source** — Read `src/cli/mod.rs` lines 2430–2460 to confirm `sessions_routine` function signature and that it has no callers other than the registry.
3. **Rename function** — In `src/cli/mod.rs`, rename `fn sessions_routine` → `fn projects_routine`. No other changes.
4. **Update registry** — In `src/cli_main.rs`, change `".sessions" => cli::sessions_routine` → `".projects" => cli::projects_routine`.
5. **Update YAML** — In `unilang.commands.yaml`, find the `.sessions` block: rename `name: .sessions` → `name: .projects`; update the `description`, `long_description`, and all `examples` that reference `.sessions` by name.
6. **Rename test file (sessions_command_test.rs)** — Copy contents; delete old file; create `tests/projects_command_test.rs`. Update: module-level doc comment, all `".sessions"` command argument strings → `".projects"`, function names that include "sessions" (optional, but preferred for clarity).
7. **Rename test file (sessions_output_format_test.rs)** — Same process → `tests/projects_output_format_test.rs`. Update all `.sessions` references.
8. **Verify compile** — `RUSTFLAGS="-D warnings" cargo check` → 0 errors, 0 warnings.
9. **Verify tests** — `cargo nextest run` → 0 failures. ALL previously-passing tests must still pass.
10. **Update `tests/readme.md`** — Rename both rows: `sessions_command_test.rs` → `projects_command_test.rs`, `sessions_output_format_test.rs` → `projects_output_format_test.rs`.
11. **Update `docs/cli/commands.md`** — Rename `### Command :: 7. .sessions` → `### Command :: 7. .projects` (numbering depends on whether tasks 013 and 014 ran first; use actual current number); update all syntax examples from `claude_storage .sessions` → `claude_storage .projects`; update all prose references.
12. **Rename `docs/cli/testing/command/sessions.md`** → `projects.md`. Update internal `.sessions` references.
13. **Update `docs/cli/testing/command/readme.md`** — Rename the `sessions.md` row → `projects.md`.
14. **Update `docs/cli/dictionary.md`** — In `Active Session` entry, change any reference to "`clg .sessions`" → "`clg .projects`". In `Scope` entry, change "in `.sessions`" → "in `.projects`".
15. **Run full validation** — `w3 .test level::3` → 0 failures.
16. **Verify no residual .sessions in production paths** — `grep -rn '\.sessions' src/ tests/ unilang.commands.yaml docs/cli/` (historical fix notes inside comments are acceptable; production command strings are not).
17. **Walk Validation Checklist** — every answer must be YES.
18. **Update task status** — Set ✅ in `task/readme.md`, move to `task/completed/`.

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `clg .projects` (bare) | project with sessions under cwd | Summary mode: `Active session` header — identical to previous `clg .sessions` output |
| `clg .projects scope::relevant` | workspace with ancestor projects | List mode: session families grouped by project |
| `clg .projects scope::global agent::1 min_entries::50` | global storage | Filtered agent session list |
| `clg .sessions` (old name) | any | Command not found — exit 1 |
| `clg .projects verbosity::1` | project with sessions | Summary mode (verbosity::1 stays in summary — regression guard for task 012 fix) |
| `grep '".projects"' src/cli_main.rs` | source | Returns 1 match |
| `grep '".sessions"' src/cli_main.rs` | source | Returns 0 matches |

## Acceptance Criteria

- `fn projects_routine` is present in `src/cli/mod.rs`; `fn sessions_routine` is absent
- `".projects"` is in `src/cli_main.rs` phf_map; `".sessions"` is absent
- `tests/projects_command_test.rs` exists; `tests/sessions_command_test.rs` is deleted
- `tests/projects_output_format_test.rs` exists; `tests/sessions_output_format_test.rs` is deleted
- `docs/cli/commands.md` Command section heading reads `.projects`
- `docs/cli/dictionary.md` `Active Session` entry references `.projects` not `.sessions`
- All previously-passing tests still pass: `w3 .test level::3` → 0 failures
- `it47` (verbosity guard) and `it1` (summary mode) tests pass in `tests/projects_command_test.rs`

## Validation

### Checklist

Desired answer for every question is YES.

**Rename complete in source**
- [ ] Is `fn projects_routine` present in `src/cli/mod.rs`?
- [ ] Is `fn sessions_routine` absent from `src/cli/mod.rs`?
- [ ] Is `".projects"` in `src/cli_main.rs` phf_map?
- [ ] Is `".sessions"` absent from `src/cli_main.rs` phf_map?

**Rename complete in YAML**
- [ ] Is `name: .projects` present in `unilang.commands.yaml`?
- [ ] Is `name: .sessions` absent from `unilang.commands.yaml`?

**Rename complete in tests**
- [ ] Does `tests/projects_command_test.rs` exist?
- [ ] Does `tests/sessions_command_test.rs` not exist?
- [ ] Does `tests/projects_output_format_test.rs` exist?
- [ ] Does `tests/sessions_output_format_test.rs` not exist?

**Rename complete in docs**
- [ ] Does `docs/cli/commands.md` use `.projects` in the command section heading?
- [ ] Does `docs/cli/dictionary.md` reference `.projects` instead of `.sessions` in `Active Session` and `Scope` entries?
- [ ] Does `docs/cli/testing/command/projects.md` exist?
- [ ] Does `docs/cli/testing/command/sessions.md` not exist?

**Behavior unchanged**
- [ ] Does `clg .projects verbosity::1` produce summary mode (not list mode)?
- [ ] Do all previously-passing tests still pass?

**Out of Scope confirmation**
- [ ] Are `.session.dir`, `.session.ensure`, and `.exists` routines unchanged?
- [ ] Is output format unchanged (no behavioral differences, only command name)?

### Measurements

**M1 — function renamed**
Command: `grep -c "fn sessions_routine\|fn projects_routine" src/cli/mod.rs`
Before: `1` (`sessions_routine`). Expected: `1` (`projects_routine`). Deviation: `0` (deleted) or `2` (duplicate).

**M2 — registry updated**
Command: `grep '"\.sessions"\|"\.projects"' src/cli_main.rs`
Before: 1 line with `.sessions`. Expected: 1 line with `.projects`, 0 with `.sessions`. Deviation: any other result.

**M3 — it47 (verbosity guard) still passes**
Command: `cargo nextest run --test projects_command_test it47 2>&1 | tail -3`
Before: passes as `sessions_command_test`. Expected: `1 passed`. Deviation: test failure means verbosity guard was accidentally broken.

**M4 — test suite passes**
Command: `w3 .test level::3 2>&1 | grep "^Summary:"`
Expected: `Summary: 13/13 crates passed, 0 failed`. Deviation: any failure.

### Invariants

- [ ] I1 — test suite: `w3 .test level::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check` → 0 warnings

### Anti-faking checks

**AF1 — tests actually use `.projects` argument (not old name)**
Check: `grep -c '".projects"' tests/projects_command_test.rs`
Expected: ≥ 10. Why: ensures the test file was updated, not just renamed with old command strings still inside.

**AF2 — no aliasing in code**
Check: `grep -rn "sessions_routine\|\.sessions" src/`
Expected: 0 matches (no fallback alias, no reference to old name in source). Why: ensures rename is complete, not shimmed.

**AF3 — old test file truly gone**
Check: `ls tests/sessions_command_test.rs 2>/dev/null | wc -l`
Expected: `0`. Why: ensures the old file was deleted, not just accompanied by the new one.

## Outcomes


# Remove deprecated `.show.project` command

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** ✅ (Complete)

## Goal

Delete the deprecated `.show.project` command and all its test infrastructure, reducing the command count from 13 to 12 and leaving no dead code. (Motivated: `.show.project` is officially deprecated with a documented migration to `.show`; it is a dead stub with no callers and its continued presence wastes test infrastructure and confuses the command surface; Observable: `show_project_routine` absent from `src/cli/mod.rs`, test file deleted, command count 12 in all docs; Scoped: only `.show.project` infrastructure deleted — no other routines touched; Testable: `grep -c "show_project_routine" src/cli/mod.rs` → `0` AND `cargo nextest run` → 0 failures.)

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs` § `show_project_routine` (line 1320) — delete function and all helpers it calls exclusively
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli_main.rs` lines 29, 41 — remove `.show.project` registry entry and `#[allow(deprecated)]` if it becomes unreferenced
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/unilang.commands.yaml` — remove `.show.project` command block
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/show_project_command.rs` — delete entire file
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/readme.md` — remove `show_project_command.rs` row
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/commands.md` — remove Command 9 section, renumber 10–13 → 9–12, update footer to `12 commands`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/readme.md` — update Implementation Status to `12/12 commands; 0 deprecated`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/testing/command/show_project.md` — delete this test doc (all its tests cover only the removed command)
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/testing/command/readme.md` — remove `show_project.md` row, update count

## Out of Scope

- Changes to `.show` command (the documented replacement) — it requires no changes
- Documentation outside `docs/cli/` — no cross-references to `.show.project` in other doc areas
- Binary rebuild / install — covered in a separate install task if needed

## Description

`.show.project` was deprecated and superseded by `.show project::PATH`. It is the only deprecated command in the codebase. The YAML definition has `status: deprecated` and the handler delegates entirely to `.show`. No unique functionality exists.

The `#[allow(deprecated)]` attribute in `src/cli_main.rs` is present to suppress compiler warnings about the deprecated entry. After deletion it may become unnecessary — verify whether any other deprecated symbol requires it before removing.

The test file `tests/show_project_command.rs` covers only deprecated functionality. Deleting it reduces test maintenance burden without losing any coverage of live functionality. `docs/cli/testing/command/show_project.md` similarly documents only the deprecated command and can be removed.

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- Feature doc `feature/001_cli_tool.md` states "backward compatibility is a non-goal" — deletion without a compatibility shim is the correct approach
- No backup files: delete completely; git history preserves recovery path
- All helper functions exclusive to `show_project_routine` must also be deleted; shared helpers must not be deleted

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note `code_design.rulebook.md` on deletion vs archive.
2. **Read source** — Read `src/cli/mod.rs` lines 1300–1380 to understand `show_project_routine` and any helpers it calls. Grep for each helper name to verify exclusivity before deleting.
3. **Delete routine** — Remove `show_project_routine` and all exclusive helpers from `src/cli/mod.rs`. Verify: `grep -c "show_project" src/cli/mod.rs` → `0`.
4. **Update registry** — Remove `".show.project" => cli::show_project_routine,` from `src/cli_main.rs` phf_map. Verify the `#[allow(deprecated)]` is still needed by other entries; if not, remove it too.
5. **Update YAML** — Remove the entire `.show.project` command block from `unilang.commands.yaml`. Verify: `grep -c "show.project" unilang.commands.yaml` → `0`.
6. **Delete test file** — Delete `tests/show_project_command.rs` entirely.
7. **Verify compile** — `RUSTFLAGS="-D warnings" cargo check` → 0 errors, 0 warnings.
8. **Verify tests** — `cargo nextest run` → 0 failures.
9. **Update `tests/readme.md`** — Remove the `show_project_command.rs` row from the Responsibility Table.
10. **Update `docs/cli/commands.md`** — Remove Command 9 (`.show.project`) section; renumber Commands 10–13 → 9–12; update footer to `**Total:** 12 commands (12 stable ✅, 0 deprecated)`.
11. **Update `docs/cli/readme.md`** — Change Implementation Status line to `**Implementation Status:** 100% (12/12 commands implemented; 0 deprecated)`.
12. **Delete `docs/cli/testing/command/show_project.md`** — Remove test doc for the deleted command.
13. **Update `docs/cli/testing/command/readme.md`** — Remove `show_project.md` row; update count.
14. **Run full validation** — `w3 .test level::3` → 0 failures, 0 warnings.
15. **Walk Validation Checklist** — every answer must be YES.
16. **Update task status** — Set ✅ in `task/readme.md`, move file to `task/completed/`.

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `clg .show.project` | any storage | Command not found error — exit 1 (command removed from registry) |
| `grep -c "show_project_routine" src/cli/mod.rs` | source file | Returns `0` (function deleted) |
| `cargo nextest run` | full test suite | All tests pass — no tests reference deleted command |

## Acceptance Criteria

- `show_project_routine` is absent from `src/cli/mod.rs` (grep → `0`)
- `.show.project` is absent from `src/cli_main.rs` phf_map
- `.show.project` block is absent from `unilang.commands.yaml`
- `tests/show_project_command.rs` is deleted
- `docs/cli/commands.md` footer reads `12 commands (12 stable ✅, 0 deprecated)`
- `docs/cli/readme.md` Implementation Status reads `12/12 commands; 0 deprecated`
- `docs/cli/testing/command/show_project.md` is deleted
- All tests pass: `w3 .test level::3` → 0 failures

## Validation

### Checklist

Desired answer for every question is YES.

**Routine deleted**
- [ ] Is `show_project_routine` absent from `src/cli/mod.rs`?
- [ ] Is `.show.project` absent from the phf_map in `src/cli_main.rs`?
- [ ] Is the `.show.project` block absent from `unilang.commands.yaml`?

**Test infrastructure removed**
- [ ] Is `tests/show_project_command.rs` deleted?
- [ ] Is `show_project_command.rs` absent from `tests/readme.md`?
- [ ] Is `docs/cli/testing/command/show_project.md` deleted?

**Documentation updated**
- [ ] Does `docs/cli/commands.md` footer read `12 commands (12 stable ✅, 0 deprecated)`?
- [ ] Does `docs/cli/readme.md` read `12/12 commands; 0 deprecated`?

**No regressions**
- [ ] Does `w3 .test level::3` report 0 failures?

**Out of Scope confirmation**
- [ ] Is `.show` routine unchanged?
- [ ] Are no other routines modified?

### Measurements

**M1 — routine absent from source**
Command: `grep -c "show_project_routine\|fn show_project" src/cli/mod.rs`
Before: `2` (function definition + registry reference). Expected: `0`. Deviation: non-zero means deletion incomplete.

**M2 — YAML command count reduced**
Command: `grep -c "^  name:" unilang.commands.yaml`
Before: `13`. Expected: `12`. Deviation: still 13 means YAML not updated.

**M3 — test suite passes**
Command: `w3 .test level::3 2>&1 | grep "^Summary:"`
Before: `Summary: 13/13 crates passed`. Expected: same (0 crates failed). Deviation: any failure.

### Invariants

- [ ] I1 — test suite: `w3 .test level::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check` → 0 warnings

### Anti-faking checks

**AF1 — registry truly cleaned (not just commented)**
Check: `grep -c "show_project" src/cli_main.rs`
Expected: `0`. Why: ensures the deprecated handler is not merely commented out.

**AF2 — no backup file created**
Check: `ls src/cli/mod.rs.bak src/cli/old_mod.rs 2>/dev/null | wc -l`
Expected: `0`. Why: ensures no archive file was created in violation of no-backup policy.

## Outcomes


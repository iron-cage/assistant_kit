# Remove duplicate `.session` command (deduplicate with `.exists`)

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** ✅ (Complete)

## Goal

Delete the `.session` command from all code, test, and documentation layers, leaving `.exists` as the sole history-check command and reducing the command count from 12 to 11. (Motivated: `.session` and `.exists` are semantically identical — both exit `0` when history exists and `1` when absent — and the one-character name difference from `.sessions` is a persistent tab-completion hazard; Observable: `session_routine` absent from `src/cli/mod.rs`, `".session"` absent from `src/cli_main.rs`, test file deleted, `.exists` note updated; Scoped: only `session_routine` and its exclusive helpers deleted, `.session.dir` and `.session.ensure` untouched; Testable: `grep -c "fn session_routine" src/cli/mod.rs` → `0` AND `cargo nextest run` → 0 failures.)

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs` § `session_routine` (line 1839) — delete function and all helpers it calls exclusively
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli_main.rs` — remove `".session" => cli::session_routine,` entry
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/unilang.commands.yaml` — remove `.session` command block
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/session_command_test.rs` — delete entire file
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/readme.md` — remove `session_command_test.rs` row
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/commands.md` — remove Command 7 section, renumber 8–12 → 7–11, update footer to `11 commands`, add note to `.exists` mentioning it replaced `.session`, update `.sessions`/`.projects` note about distinct-from
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/readme.md` — update Implementation Status to `11/11 commands`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/testing/command/session.md` — delete this test doc
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/testing/command/readme.md` — remove `session.md` row, update count
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/dictionary.md` § `Active Session` — update reference from `.sessions` (or `.session`) to ensure accuracy

## Out of Scope

- `.session.dir` and `.session.ensure` — these are NOT duplicates; they compute paths and create directories. Untouched.
- `.exists` implementation changes — its behavior is already correct; only its documentation note is updated
- Binary rebuild / install

## Description

`.session` and `.exists` are semantically identical. Both take `path::` and `topic::` parameters, both exit `0` when history exists, `1` when absent, and `2` on error. The only documented difference is that `.exists` has "explicit exit-code documentation making it ideal for scripting." That difference belongs in prose, not in a separate command.

The one-character name difference from `.sessions` is a persistent usability hazard. Tab completion offers `.session`, `.session.dir`, `.session.ensure`, and `.sessions` — four entries that differ by one suffix. Removing `.session` reduces this to three distinct `.session*` entries and eliminates the most confusing one (the one that silently checks existence when the user likely meant `.sessions`).

Before deleting `session_routine`, verify that any helper function it calls is also called by `exists_routine` or other routines. Do not delete shared helpers.

## Requirements

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- Verify shared helpers before deleting: `session_routine` and `exists_routine` may call the same core `check_continuation` function — confirm shared functions are NOT deleted
- Feature doc states "backward compatibility is a non-goal" — no compatibility alias
- No backup files — delete completely

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note `code_design.rulebook.md` on deletion scope.
2. **Read both routines** — Read `src/cli/mod.rs` lines 1839–1880 (`session_routine`) and the `exists_routine` (search for `fn exists_routine`). List every helper each calls; identify shared vs exclusive helpers.
3. **Delete routine** — Remove `session_routine` and all EXCLUSIVE helpers from `src/cli/mod.rs`. Do NOT delete any function also called by `exists_routine`. Verify: `grep -c "fn session_routine" src/cli/mod.rs` → `0`.
4. **Update registry** — Remove `".session" => cli::session_routine,` from `src/cli_main.rs` phf_map. Verify: `grep -c '"\.session"' src/cli_main.rs` → `0` AND `grep -c '"\.sessions"' src/cli_main.rs` → `1`.
5. **Update YAML** — Remove the `.session` command block from `unilang.commands.yaml`. Verify: `grep -c "name: .session$" unilang.commands.yaml` → `0`.
6. **Delete test file** — Delete `tests/session_command_test.rs` entirely.
7. **Verify compile** — `RUSTFLAGS="-D warnings" cargo check` → 0 errors, 0 warnings.
8. **Verify tests** — `cargo nextest run` → 0 failures.
9. **Update `tests/readme.md`** — Remove the `session_command_test.rs` row.
10. **Update `docs/cli/commands.md`** — Remove Command 7 (`.session`) section; renumber 8–12 → 7–11 (note: if task 013 ran first, renumber starting from the current max); update footer to `**Total:** 11 commands (11 stable ✅)`; in the `.exists` Notes section add: "This is the sole history-check command; `.session` was removed as a duplicate."; in the `.sessions`/`.projects` Notes section update the cross-reference "Distinct from `.session` (singular)" → "Distinct from `.exists`: that command checks existence; this lists sessions."
11. **Update `docs/cli/readme.md`** — Change Implementation Status to `**Implementation Status:** 100% (11/11 commands implemented)`.
12. **Delete `docs/cli/testing/command/session.md`** — Remove test doc for deleted command.
13. **Update `docs/cli/testing/command/readme.md`** — Remove `session.md` row; update count.
14. **Run full validation** — `w3 .test level::3` → 0 failures.
15. **Walk Validation Checklist** — every answer must be YES.
16. **Update task status** — Set ✅ in `task/readme.md`, move to `task/completed/`.

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `clg .session` | any storage | Command not found error — exit 1 (command removed from registry) |
| `clg .exists` (bare) | project with history | Exit 0, `"sessions exist\n"` on stdout — `.exists` behavior unchanged |
| `clg .exists path::PATH` | valid path with history | Exit 0 — `.exists` with path param still works |
| `clg .sessions` (bare) | project under cwd | Summary mode output — `.sessions` unaffected by deletion |
| `clg .session.dir path::.` | any | Correct path output — `.session.dir` unaffected |
| `grep -c "fn session_routine" src/cli/mod.rs` | source | Returns `0` |

## Acceptance Criteria

- `session_routine` is absent from `src/cli/mod.rs`
- `".session"` is absent from `src/cli_main.rs` phf_map
- `.session` block is absent from `unilang.commands.yaml`
- `tests/session_command_test.rs` is deleted
- `.exists` Notes section in `docs/cli/commands.md` mentions it replaced `.session`
- `docs/cli/commands.md` footer reads `11 commands (11 stable ✅)`
- All tests pass: `w3 .test level::3` → 0 failures
- `exists_routine` and all `.session.dir`/`.session.ensure` routines are unmodified

## Validation

### Checklist

Desired answer for every question is YES.

**Routine deleted**
- [ ] Is `session_routine` absent from `src/cli/mod.rs`?
- [ ] Is `".session"` absent from `src/cli_main.rs` phf_map?
- [ ] Is the `.session` block absent from `unilang.commands.yaml`?

**Related commands unaffected**
- [ ] Is `exists_routine` still present in `src/cli/mod.rs`?
- [ ] Is `session_dir_routine` still present?
- [ ] Is `session_ensure_routine` still present?
- [ ] Is `".sessions"` still in `src/cli_main.rs` phf_map?

**Test infrastructure removed**
- [ ] Is `tests/session_command_test.rs` deleted?
- [ ] Is `session_command_test.rs` absent from `tests/readme.md`?
- [ ] Is `docs/cli/testing/command/session.md` deleted?

**Documentation updated**
- [ ] Does `docs/cli/commands.md` footer read `11 commands (11 stable ✅)`?
- [ ] Does `.exists` Notes mention "sole history-check command" and removal of `.session`?
- [ ] Does `docs/cli/readme.md` read `11/11 commands`?

**No regressions**
- [ ] Does `w3 .test level::3` report 0 failures?

**Out of Scope confirmation**
- [ ] Are `.session.dir` and `.session.ensure` routines unchanged?
- [ ] Is `exists_routine` unchanged?

### Measurements

**M1 — routine absent**
Command: `grep -c "fn session_routine" src/cli/mod.rs`
Before: `1`. Expected: `0`. Deviation: non-zero means deletion incomplete.

**M2 — YAML command count**
Command: `grep -c "^  name:" unilang.commands.yaml`
Before: `12` (after task 013). Expected: `11`. Deviation: still 12 means YAML not updated.

**M3 — `.exists` still registered**
Command: `grep -c '".exists"' src/cli_main.rs`
Before: `1`. Expected: `1`. Deviation: `0` means `.exists` was accidentally removed.

**M4 — test suite passes**
Command: `w3 .test level::3 2>&1 | grep "^Summary:"`
Expected: `Summary: 13/13 crates passed, 0 failed`. Deviation: any failure.

### Invariants

- [ ] I1 — test suite: `w3 .test level::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check` → 0 warnings

### Anti-faking checks

**AF1 — `.exists` untouched**
Check: `grep -c "fn exists_routine" src/cli/mod.rs`
Expected: `1`. Why: confirms the surviving history-check command was not accidentally modified.

**AF2 — registry correctly differentiates `.session` variants**
Check: `grep '"\.session' src/cli_main.rs`
Expected: lines for `.session.dir` and `.session.ensure` only — no bare `".session"`. Why: ensures only the exact duplicate was removed, not its dotted variants.

## Outcomes


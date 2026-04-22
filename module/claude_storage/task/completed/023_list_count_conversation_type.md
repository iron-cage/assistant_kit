# Extend `.list` and `.count` with `type::conversation` and count mode

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** ✅ (Complete)

## Goal

Add `type::conversation` to `.list` and `target::conversations` to `.count`, and introduce a `count::` boolean mode on `.list` that outputs the item count instead of the full listing (Motivated: tasks 021/022 introduce `Conversation` as the user-facing concept but the CLI commands still only enumerate sessions — users can neither list conversations of a project via `.list` nor count them directly; without these additions the taxonomy document and dictionary entries promise a capability that the CLI doesn't deliver; Observable: `clg .list type::conversation project::<id>` prints one conversation identifier per line, `clg .list type::conversation count::1 project::<id>` prints a bare integer, `clg .count target::conversations project::<id>` prints a bare integer, and `params.md` and `commands.md` are updated to document the new values; Scoped: `src/cli/mod.rs` `list_routine` and `count_routine`, YAML command definitions, `docs/cli/commands.md`, `docs/cli/params.md`; Testable: `w3 .test level::3` passes and `grep -c "type::conversation" tests/list_command_test.rs` returns ≥ 1).

**Prerequisite:** Task 021 (`Conversation` type introduced) and Task 022 (conversation terminology in place).

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs`
  - `list_routine`: add `type::conversation` branch — requires `project::` parameter, iterates `group_into_conversations` result, outputs one conversation ID per line; add `count::` boolean — when true, output the count integer instead of the list
  - `count_routine`: add `target::conversations` branch — requires `project::` parameter, returns `group_into_conversations` length
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/commands/*.yml` or equivalent YAML command definition for `.list` and `.count` — register new parameter values
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/commands.md`
  - Add `type::conversation` row to `.list` parameter table
  - Add `count::` parameter row to `.list` parameter table
  - Add `target::conversations` row to `.count` parameter table
  - Add example invocations in the §`.list` and §`.count` sections
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/docs/cli/params.md`
  - Add `count::` entry (boolean, defaults false, controls output mode)
  - Update `type::` entry to enumerate `conversation` as a valid value
  - Update `target::` entry to enumerate `conversations` as a valid value
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/list_command_test.rs` — add tests per Test Matrix
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/count_command_test.rs` — add `it_count_conversations_requires_project` and `it_count_target_conversations`

## Out of Scope

- `type::entry` for `.list` (listing individual JSONL entries via CLI) — future task
- Changing the default behavior of `.list` when no `type::` specified
- Changes to `.projects`, `.show`, `.search`, `.export` commands
- Implementing actual cross-session conversation chain detection (→ future task, uses 1:1 mapping from task 021)

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   `type::conversation` requires a `project::` parameter; error clearly if absent
-   `count::` mode must work with any `type::` value (not just `conversation`)
-   `group_into_conversations` must be called from task 021's implementation — do not re-implement
-   Output format for count mode: bare integer with newline, no label

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note CLI parameter naming conventions and codestyle rules.
2. **Read documentation** — Read `docs/cli/commands.md` §`.list` and §`.count`; read `docs/cli/params.md` §`type::` and §`target::`; read `docs/cli/dictionary.md` §`Conversation`; read `docs/claude_code/007_concept_taxonomy.md` for the authoritative Conversation definition.
3. **Read source** — Read `src/cli/mod.rs` `list_routine` (line ~516) and `count_routine` (line ~1371); understand current type/target dispatch and error handling patterns.
4. **Write failing tests** — Add `it_list_type_conversation_requires_project` and `it_list_count_mode_outputs_integer` to `tests/list_command_test.rs`; add `it_count_target_conversations` to `tests/count_command_test.rs`. Run `w3 .test level::3` and confirm new tests fail.
5. **Implement** —
   a. Add `type::conversation` branch in `list_routine`: get project param (error if absent), load project, call `group_into_conversations`, format output; when `count::` true, output only integer.
   b. Add `count::` boolean handling in `list_routine` for all type branches.
   c. Add `target::conversations` branch in `count_routine`: get project param (error if absent), load project, call `group_into_conversations`, return length.
   d. Register new parameter values in YAML command definitions.
6. **Update docs** — Update `commands.md` and `params.md` per In Scope above.
7. **Green state** — `w3 .test level::3` passes with zero failures and zero warnings.
8. **Walk Validation Checklist** — every answer must be YES.
9. **Update task status** — ✅ in `task/readme.md`, move to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `.list type::conversation project::<valid-id>` | project with ≥1 session | Outputs ≥1 conversation IDs |
| T02 | `.list type::conversation` (no project param) | missing required param | Returns error: "project parameter required for listing conversations" |
| T03 | `.list type::conversation count::1 project::<valid-id>` | count mode | Outputs a bare integer ≥ 1 |
| T04 | `.list type::all count::1` | count mode on default type | Outputs a bare integer (project count) |
| T05 | `.count target::conversations project::<valid-id>` | conversations count | Outputs an integer ≥ 1 |
| T06 | `.count target::conversations` (no project param) | missing required param | Returns error: "project parameter required for counting conversations" |

## Acceptance Criteria

- `grep -c "type::conversation" tests/list_command_test.rs` returns ≥ 1
- `grep -c "target::conversations" tests/count_command_test.rs` returns ≥ 1
- `clg .list type::conversation project::<existing-id>` exits 0 with ≥1 line of output
- `clg .list type::conversation count::1 project::<existing-id>` exits 0 with a bare integer
- `clg .count target::conversations project::<existing-id>` exits 0 with a bare integer
- `params.md` contains `count::` entry with boolean type and false default

## Validation

### Checklist

Desired answer for every question is YES.

**`.list type::conversation`**
- [ ] C1 — Does `.list type::conversation project::<id>` output conversation identifiers (one per line)?
- [ ] C2 — Does `.list type::conversation` without project parameter return a clear error?
- [ ] C3 — Does `.list type::conversation count::1 project::<id>` output only a bare integer?

**`.count target::conversations`**
- [ ] C4 — Does `.count target::conversations project::<id>` output a bare integer?
- [ ] C5 — Does `.count target::conversations` without project parameter return a clear error?

**Documentation**
- [ ] C6 — Does `commands.md` §`.list` show `type::conversation` and `count::` in the parameter table?
- [ ] C7 — Does `commands.md` §`.count` show `target::conversations` in the parameter table?
- [ ] C8 — Does `params.md` have a `count::` entry?

**Out of Scope confirmation**
- [ ] C9 — Is `type::entry` not yet implemented in `.list`?
- [ ] C10 — Are `.projects`, `.show`, `.search`, `.export` unchanged?

### Measurements

- [ ] M1 — test count: `w3 .test level::3 2>&1 | grep "test result"` → `test result: ok. 294 passed` (was: 291 after tasks 021+022; +3 new tests)

### Invariants

- [ ] I1 — test suite: `w3 .test level::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --all-features` → 0 warnings

### Anti-faking checks

**AF1 — Verify conversation branch in list_routine**
Check: `grep -n "type::conversation\|\"conversation\"" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs | grep -i "list\|type"`
Expected: at least one match inside `list_routine`. Why: catches if change was made in a comment or dead branch.

**AF2 — Verify count mode parameter**
Check: `grep -n "count" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs | grep "get_boolean\|\"count\""`
Expected: at least one match in `list_routine`. Why: ensures count mode is wired to list output path, not just parsed.

## Outcomes

[Added upon task completion]

# Fix BUG-325 — Extend dot04 Visible Array to 18 Commands

## Execution State

- **Executor Type:** any
- **filed_by:** agent
- **actor:** null
- **started_at:** 2026-07-02
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** BUG-325
- **dir:** tests/cli/
- **validated_by:** agent (MAAV 3-subagent dispatch 2026-07-02)
- **validation_date:** 2026-07-02

## Goal

Add `".models"` and `".model.select"` to the `visible` array in `dot04_all_visible_commands_present`,
extending from 16 to 18 entries so the test reflects the current set of visible commands registered
in `register_commands()`. **Why now:** Features 068 and 069 added `.models` and `.model.select`
to the live command registry and updated `dot05_exactly_eighteen_command_rows` to assert count == 18,
but the `dot04` visible array was not updated — leaving it asserting only 16 of the 18 present
commands. The test would pass incorrectly on a registry with fewer commands but never catch a missing
`.models` or `.model.select`. Observable end-state: `dot04` asserts all 18 visible commands by name;
`dot05` asserts exactly 18 rows; both pass under `w3 .test level::3`.

## In Scope

- `tests/cli/dot_test.rs` — `dot04_all_visible_commands_present`: insert `".models"` and
  `".model.select"` immediately after `".model"` in the `visible` array

## Out of Scope

- Source code changes to command registration
- Changes to `dot05_exactly_eighteen_command_rows` (already correct)
- Any other test files

## Verification Record

MAAV 3-subagent dispatch (2026-07-02):

| Agent | Role | Verdict |
|-------|------|---------|
| Conformance | Validator | PASS — M1: `".models"` present (line 116); M2: `".model.select"` present (line 117); I1: no stale 16-count assertion; I2: no `#[ignore]`; I3: both entries un-commented |
| Adversarial | Adversary | PASS — ordering consistent with hierarchical (not strict-alpha) convention; test matrix and section comment both say 18; stale "16 commands" in `vision.md` and `docs/cli/command/readme.md` found and fixed |
| Anti-Cheat | Validator | PASS — 18 real command names in array; assertion loop uses real binary output (`run_cs`) |

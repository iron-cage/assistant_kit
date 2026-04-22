# Introduce `Conversation` type and conversation chain detection algorithm

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** ✅ (Complete)

## Goal

Introduce `Conversation` as a first-class named type in `src/cli/mod.rs` and implement the initial conversation chain detection algorithm (Motivated: tasks 022 and 023 both require a `Conversation` abstraction to display conversations as the primary unit rather than sessions; without this foundation they cannot proceed; Observable: a `Conversation` struct exists in `src/cli/mod.rs` wrapping one `SessionFamily`, a `group_into_conversations` function groups a `Vec<SessionFamily>` into `Vec<Conversation>`, and the existing `projects` display code uses `Conversation` instead of `SessionFamily` as its iteration unit; Scoped: changes are confined to `src/cli/mod.rs` internal types; no CLI output changes; Testable: `w3 .test level::3` passes and `grep -c "struct Conversation" src/cli/mod.rs` returns 1).

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs`
  - `struct Conversation { families: Vec<SessionFamily> }` — a conversation is a logical grouping of one or more session families; initial implementation: one family per conversation
  - `fn group_into_conversations(families: Vec<SessionFamily>) -> Vec<Conversation>` — chain detection algorithm; initial implementation: identity mapping (each family = one conversation)
  - Refactor internal `projects` rendering code to iterate over `Vec<Conversation>` instead of `Vec<SessionFamily>` directly (no output change)
  - `impl Conversation` — helper methods: `fn root_session(&self) -> Option<&Session>`, `fn all_agents(&self) -> Vec<&AgentInfo>`, `fn conversation_count(&self) -> usize`
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/projects_command_test.rs` — add test `it_conversation_groups_families_one_to_one` verifying the 1:1 mapping invariant

## Out of Scope

- Changing any CLI output format or output text (→ task 022)
- Detecting actual continuation chains across multiple session files (future enhancement; algorithm is defined but identity-mapped for now)
- Adding new CLI parameters or commands (→ task 023)
- Changes to `claude_storage_core` (the storage layer)

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   `Conversation` must be defined and used only within `src/cli/mod.rs` (CLI layer); do not leak it into the public API of `claude_storage_core`
-   `group_into_conversations` must be pure — no I/O, no side effects; takes owned input, returns owned output
-   Initial implementation must pass all 289+ existing tests unchanged
-   `Conversation` struct must carry a `doc` comment explaining the future chain detection contract

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note constraints on struct naming, function length (≤50 lines), and module design.
2. **Read source** — Read `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs` §§ `SessionFamily`, `AgentInfo`, `aggregate_projects`, `render_projects_list`, `render_project_families` to understand the current call chain.
3. **Read taxonomy doc** — Read `/home/user1/pro/lib/wip_core/claude_tools/dev/docs/claude_code/007_concept_taxonomy.md` for authoritative definition of Conversation and the chain detection contract.
4. **Write Test Matrix** — populate matrix below before opening any test file.
5. **Write failing tests** — in `tests/projects_command_test.rs`, add test `it_conversation_groups_families_one_to_one` confirming each SessionFamily becomes exactly one Conversation. Run `w3 .test level::3` and confirm the test fails with a compile error (type not yet defined).
6. **Implement** —
   a. Add `/// Conversation is the user-facing unit...` doc comment + `struct Conversation` after `SessionFamily`.
   b. Add `impl Conversation` with helper methods.
   c. Add `fn group_into_conversations` (pure, identity-mapped).
   d. Refactor `aggregate_projects` / `render_project_families` to route through `group_into_conversations`.
7. **Green state** — `w3 .test level::3` must pass with zero failures and zero warnings before proceeding.
8. **Refactor if needed** — ensure no function exceeds 50 lines; no duplication; all public items have `///` doc comments.
9. **Submit for Validation** — trigger SUBMIT transition (⏳ → 🔍).
10. **Update task status** — on validation pass, set ✅ in `task/readme.md`, recalculate Advisability=0, re-sort index, move file to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `group_into_conversations(vec![family_a, family_b])` | initial 1:1 implementation | Returns `vec![conv_a, conv_b]` — each family maps to exactly one conversation |
| T02 | `group_into_conversations(vec![])` | empty input | Returns `vec![]` |
| T03 | `group_into_conversations(vec![orphan_family])` | orphan family (no root session) | Returns `vec![conv_orphan]` — orphans are also single-family conversations |

## Acceptance Criteria

- `grep -c "struct Conversation" src/cli/mod.rs` returns `1`
- `grep -c "fn group_into_conversations" src/cli/mod.rs` returns `1`
- `group_into_conversations` is pure (no `impl` on `std::io` or filesystem calls inside)
- All 289+ existing `w3 .test level::3` tests pass unchanged
- New test `it_conversation_groups_families_one_to_one` in `tests/projects_command_test.rs` passes

## Validation

### Checklist

Desired answer for every question is YES.

**Conversation struct**
- [ ] C1 — Does `struct Conversation` exist in `src/cli/mod.rs`?
- [ ] C2 — Does `Conversation` have a `///` doc comment explaining the chain detection contract?
- [ ] C3 — Does `impl Conversation` include `root_session`, `all_agents`, and `conversation_count` methods?
- [ ] C4 — Is `group_into_conversations` a standalone function (not a method)?
- [ ] C5 — Is `group_into_conversations` pure (no filesystem I/O)?

**Integration**
- [ ] C6 — Does the projects rendering code iterate over `Vec<Conversation>` (not `Vec<SessionFamily>` directly)?
- [ ] C7 — Is CLI output identical before and after this change (no format regressions)?

**Out of Scope confirmation**
- [ ] C8 — Is the `SessionFamily` struct unchanged (no added/removed fields)?
- [ ] C9 — Are `params.md` and `commands.md` unchanged?

### Measurements

- [ ] M1 — test count: `w3 .test level::3 2>&1 | grep "test result"` → `test result: ok. 290 passed` (was: 289 before adding T01 test)

### Invariants

- [ ] I1 — test suite: `w3 .test level::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --all-features` → 0 warnings

### Anti-faking checks

**AF1 — Verify Conversation struct exists**
Check: `grep "struct Conversation" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs`
Expected: one match with `struct Conversation`. Why: catches if the implementation used a type alias instead of a named struct.

**AF2 — Verify group_into_conversations is pure**
Check: `grep -A 20 "fn group_into_conversations" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs | grep "std::fs\|std::io\|File::"`
Expected: no matches. Why: ensures the function has no side effects.

## Outcomes

[Added upon task completion]

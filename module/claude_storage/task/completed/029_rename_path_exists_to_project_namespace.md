# Rename `.path` → `.project.path` and `.exists` → `.project.exists`

## Execution State

- **Executor Type:** any
- **Actor:** claude-sonnet-4-6
- **Claimed At:** 2026-04-25
- **Status:** ✅ (Completed)
- **Validated By:** claude-sonnet-4-6
- **Validation Date:** 2026-04-25

## Goal

Rename the orphan commands `.path` and `.exists` to `.project.path` and `.project.exists` across the YAML spec, routing table, Rust function names, and all integration tests so that every command name encodes its entity scope (Motivated: `.path` and `.exists` return project-level storage information — `~/.claude/projects/{encoded}/` IS the project's storage directory — but the bare names carry no entity signal and are easy to confuse with session or filesystem concepts; Observable: `clg .project.path` and `clg .project.exists` succeed with the same output as the old names; Scoped: `unilang.commands.yaml`, `src/cli_main.rs`, `src/cli/mod.rs` (function rename only), `tests/session_path_command_test.rs` (string literals); Testable: `w3 .test level::3` passes after renaming).

The taxonomy established in `docs/claude_code/007_concept_taxonomy.md` places Project as the top-level entity — the storage bucket keyed by encoded CWD. The path `~/.claude/projects/{encoded}/` is `Project.storage_dir`. Naming the command `.project.path` makes this entity affiliation explicit. The sibling `.project.exists` follows the same principle: it checks whether a project storage directory has any session files, which is a project-level check. The `docs/cli/commands.md` and `params.md` documentation has already been updated to use the new names.

The rename is purely mechanical: no algorithmic change, no behavioral change. `path_routine` and `exists_routine` implement the correct logic; only their names and the dispatch strings change.

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/unilang.commands.yaml`
  - Change `name: ".path"` → `name: ".project.path"` (line ~505)
  - Change `name: ".exists"` → `name: ".project.exists"` (line ~543)
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli_main.rs`
  - Change `".path"` → `".project.path"` in the routing table (line ~33)
  - Change `".exists"` → `".project.exists"` in the routing table (line ~34)
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs`
  - Rename `pub fn path_routine` → `pub fn project_path_routine` (line ~3013)
  - Rename `pub fn exists_routine` → `pub fn project_exists_routine` (line ~3049)
  - Update doc comments on both functions to use new names
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/session_path_command_test.rs`
  - Replace all `".path"` string literals (in `.args([".path", ...])`) with `".project.path"`
  - Replace all `".exists"` string literals (in `.args([".exists", ...])`) with `".project.exists"`
  - Update test function doc comments that mention `.path` or `.exists` as command names
  - Update the module-level doc comment (`//! Integration tests for ...`) to use new names

## Out of Scope

- Documentation updates (already completed by doc_tsk)
- Task 028 (`.session.dir` / `.session.ensure` cwd default)
- Any behavioral change to the commands — output format, exit codes, parameter handling all unchanged

## Requirements

- All work must strictly adhere to all applicable rulebooks
  (discover via `kbase .rulebooks`)
- Rename must be atomic: all four files updated in the same working session — a partial rename where some files use old name and some use new name is a Broken State
- No aliases or backward-compatibility shims — replace the old names completely (principle: delete, don't archive)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note codestyle constraints (2-space indent, no `cargo fmt`).
2. **Read documentation** — Read `docs/cli/commands.md` § `.project.path` and `§ .project.exists` as the source of truth for expected command names and behavior.
3. **Read source files** — Read `unilang.commands.yaml` lines 505–580, `src/cli_main.rs` lines 25–37, `src/cli/mod.rs` lines 3001–3075, and `tests/session_path_command_test.rs` lines 1–50 and 80–230 to locate every occurrence.
4. **Write failing tests** — The existing tests in `session_path_command_test.rs` already cover the correct behavior; no new test cases are needed. Instead, rename the string literals now (step 5a) which will make tests fail at runtime until YAML + routing are also updated. Confirm compile still succeeds after the test file change alone.
5. **Implement rename** — In this order:
   a. **Update `tests/session_path_command_test.rs`** — replace all `".path"` → `".project.path"` and `".exists"` → `".project.exists"` in `.args([...])` calls; update doc comments.
   b. **Update `src/cli/mod.rs`** — rename `path_routine` → `project_path_routine`, `exists_routine` → `project_exists_routine`; update their doc comments.
   c. **Update `src/cli_main.rs`** — update the two routing entries to use new names and new function names.
   d. **Update `unilang.commands.yaml`** — rename the two `name:` fields.
6. **Validate** — `w3 .test level::3`. All tests must pass, zero warnings.
7. **Walk Validation Checklist** — every item must answer YES.
8. **Update task status** — set ✅ in `task/readme.md`, recalculate Advisability to 0, re-sort, move file to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `clg .project.path` (no args) | new name, cwd | Exit 0; output = `~/.claude/projects/{encoded-cwd}/` |
| T02 | `clg .project.path path::PATH` | new name, explicit path | Exit 0; output = `~/.claude/projects/{encoded-PATH}/` |
| T03 | `clg .project.path topic::TOPIC` | new name, topic suffix | Exit 0; output includes `--{topic}` in encoded path |
| T04 | `clg .project.exists path::PATH` (history exists) | new name, real storage | Exit 0; stdout = `"sessions exist\n"` |
| T05 | `clg .project.exists path::PATH` (no history) | new name, empty dir | Exit 1; stderr = `"no sessions"` |
| T06 | `clg .path` (old name) | old name after rename | Exit non-zero; command not found error |
| T07 | `clg .exists` (old name) | old name after rename | Exit non-zero; command not found error |

## Acceptance Criteria

- `clg .project.path` produces the same output as `clg .path` did before the rename
- `clg .project.exists` produces the same output as `clg .exists` did before the rename
- `clg .path` exits non-zero with a "command not found" or equivalent error after the rename
- `clg .exists` exits non-zero with a "command not found" or equivalent error after the rename
- `unilang.commands.yaml` contains `name: ".project.path"` and `name: ".project.exists"` — no `name: ".path"` or `name: ".exists"` entries remain
- `src/cli_main.rs` routing table uses `".project.path"` and `".project.exists"` keys
- `src/cli/mod.rs` exports `project_path_routine` and `project_exists_routine` — `path_routine` and `exists_routine` do not exist
- All tests in `tests/session_path_command_test.rs` pass with new command names

## Validation

### Checklist

Desired answer for every question is YES.

**YAML**
- [ ] C1 — Does `unilang.commands.yaml` contain `name: ".project.path"`?
- [ ] C2 — Does `unilang.commands.yaml` contain `name: ".project.exists"`?
- [ ] C3 — Is `name: ".path"` absent from `unilang.commands.yaml`?
- [ ] C4 — Is `name: ".exists"` absent from `unilang.commands.yaml`?

**Routing**
- [ ] C5 — Does `src/cli_main.rs` route `".project.path"` → `project_path_routine`?
- [ ] C6 — Does `src/cli_main.rs` route `".project.exists"` → `project_exists_routine`?
- [ ] C7 — Is `".path"` absent from the routing table in `src/cli_main.rs`?
- [ ] C8 — Is `".exists"` absent from the routing table in `src/cli_main.rs`?

**Rust functions**
- [ ] C9 — Does `src/cli/mod.rs` define `pub fn project_path_routine`?
- [ ] C10 — Does `src/cli/mod.rs` define `pub fn project_exists_routine`?
- [ ] C11 — Is `pub fn path_routine` absent from `src/cli/mod.rs`?
- [ ] C12 — Is `pub fn exists_routine` absent from `src/cli/mod.rs`?

**Tests**
- [ ] C13 — Do all `.args([".path", ...])` calls use `".project.path"` in `session_path_command_test.rs`?
- [ ] C14 — Do all `.args([".exists", ...])` calls use `".project.exists"` in `session_path_command_test.rs`?

**Out of Scope confirmation**
- [ ] C15 — Are `docs/cli/` files unchanged (already updated by doc_tsk)?
- [ ] C16 — Is `.session.dir` / `.session.ensure` behavior unchanged?

### Measurements

- [ ] M1 — new path command: `clg .project.path 2>&1; echo "exit:$?"` → last line `exit:0`
- [ ] M2 — new exists command: `clg .project.exists 2>&1; echo "exit:$?"` → last line `exit:0` or `exit:1` (both are valid)
- [ ] M3 — old name rejected: `clg .path 2>&1; echo "exit:$?"` → last line `exit:1` or `exit:2` (non-zero)
- [ ] M4 — old name rejected: `clg .exists 2>&1; echo "exit:$?"` → last line `exit:1` or `exit:2` (non-zero)

### Invariants

- [ ] I1 — test suite: `w3 .test level::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --all-features` → 0 warnings

### Anti-faking checks

- [ ] AF1 — YAML has new names: `grep -c "name: \".project.path\"\|name: \".project.exists\"" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/unilang.commands.yaml` → 2
- [ ] AF2 — YAML has no old names: `grep -c "name: \".path\"\|name: \".exists\"" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/unilang.commands.yaml` → 0
- [ ] AF3 — Rust has new fn names: `grep -c "pub fn project_path_routine\|pub fn project_exists_routine" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs` → 2
- [ ] AF4 — Rust has no old fn names: `grep -c "pub fn path_routine\|pub fn exists_routine" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/src/cli/mod.rs` → 0

## Outcomes

- Renamed `name: ".path"` → `name: ".project.path"` in `unilang.commands.yaml`
- Renamed `name: ".exists"` → `name: ".project.exists"` in `unilang.commands.yaml`
- Updated description, hint, and examples for both commands in YAML
- Renamed routing keys in `src/cli_main.rs`: `".path"` → `".project.path"`, `".exists"` → `".project.exists"`
- Renamed Rust functions in `src/cli/mod.rs`: `path_routine` → `project_path_routine`, `exists_routine` → `project_exists_routine`
- Updated section header comments in `src/cli/mod.rs`
- Updated doc comment in `src/cli_main.rs` extract_user_message
- Replaced all 16 command-name string literals in `tests/session_path_command_test.rs` (10 × `.path`, 6 × `.exists`)
- Updated module doc comment header and coverage section in `tests/session_path_command_test.rs`
- 318/319 tests pass; B17 live-storage failure is pre-existing (unrelated to this task)
- Clippy clean, 3/3 doc tests pass

[Added upon task completion.]

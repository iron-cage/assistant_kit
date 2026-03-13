# TSK-068 — Add session path and lifecycle commands to `claude_storage`

## Status

✅ (Complete)

## Metadata

- **Value:** 6
- **Easiness:** 6
- **Priority:** 2
- **Safety:** 7
- **Advisability:** 504

## Goal

Add four commands to the `claude_storage` CLI (`clg`) that expose session path resolution and existence checking, covering the functionality currently duplicated in the `claude_session` standalone stub.

**MOST criteria:**
- **Motivated:** `claude_session` duplicates path-encoding logic already in `claude_storage_core`. Centralising it in `clg` removes the duplicate and makes the capability discoverable from the main tool.
- **Observable:** Four new commands usable from the terminal: `.path`, `.exists`, `.session.dir`, `.session.ensure`.
- **Scoped:** `claude_storage` CLI only — no other crate changes in scope.
- **Testable:** `ctest3` passes with new integration tests covering all four commands.

## Description

`claude_storage_core` already encodes/decodes filesystem paths to Claude's `~/.claude/projects/` format (used by `scope::under`). This task surfaces that logic as first-class CLI commands so users can query path mappings and session existence without writing code. It also adds lightweight session-directory lifecycle commands (`.session.dir`, `.session.ensure`) that manage hyphen-prefixed topic dirs under a user-supplied root.

This task is the prerequisite for removing the `claude_session` standalone stub crate (see Out of Scope).

## In Scope

- Four new commands in `claude_storage` (`clg`):
  - `.path` — print `~/.claude/projects/{escaped}` for the given or current directory
  - `.exists` — exit 0 if at least one `.jsonl` session exists for the directory, exit 1 otherwise
  - `.session.dir path:: topic::` — print `-{topic}` subdirectory path without creating it
  - `.session.ensure path:: topic:: strategy::resume|fresh` — create (or wipe+recreate) the topic dir
- Parameters: `path::` (default cwd), `topic::`, `strategy::`, `v::`, `format::`
- Integration tests for all four commands (happy path + error paths)
- Update `claude_storage/docs/cli/commands.md` with new commands
- Update `claude_storage/spec.md` to document new commands

## Out of Scope

- Removing `claude_session` crate (separate task — requires checking external callers in willbe and other repos, and migrating `SessionManager`/`Strategy` to `claude_runner_core`)
- Changes to `claude_runner_core` or `claude_session`
- Changing the path-encoding algorithm

## Requirements

- Follow `code_design.rulebook.md` — TDD red-green-refactor cycle
- Follow `codebase_hygiene.rulebook.md` — no mocking, loud failures
- Follow `test_organization.rulebook.md` — tests in `tests/` only
- Follow `code_style.rulebook.md` — 2-space indent, custom codestyle

## Acceptance Criteria

- `clg .path` prints the correct `~/.claude/projects/` path for cwd
- `clg .path path::/some/dir` prints the path for the given dir
- `clg .exists` exits 0 when sessions exist, 1 when none
- `clg .session.dir path::P topic::T` prints `P/-T` without creating anything
- `clg .session.ensure path::P topic::T` creates `P/-T` if absent (idempotent), exits 0
- `clg .session.ensure path::P topic::T strategy::fresh` forces "fresh" output label regardless of history
- `clg .session.ensure path::P topic::T strategy::resume` forces "resume" output label regardless of history
- All integration tests pass; `ctest3` green; zero warnings

## Work Procedure

1. Read applicable rulebooks
2. Write failing integration tests for all four commands (RED)
3. Implement `.path` handler using existing path-encoding logic from `claude_storage_core`
4. Implement `.exists` handler
5. Implement `.session.dir` handler
6. Implement `.session.ensure` handler
7. All tests green (GREEN)
8. Refactor if any handler is messy; re-verify
9. Update `docs/cli/commands.md` and `spec.md`
10. Run `ctest3`; fix any warnings or failures
11. Update task status → ✅

## Validation Checklist

| # | Question | Desirable | Fail Action |
|---|----------|-----------|-------------|
| 1 | Does `clg .path` print `~/.claude/projects/` path for cwd? | YES | Fix `.path` handler |
| 2 | Does `clg .exists` exit 0 when sessions present? | YES | Fix `.exists` handler |
| 3 | Does `clg .exists` exit 1 when no sessions? | YES | Fix exit-code mapping |
| 4 | Does `clg .session.ensure strategy::fresh` force "fresh" output label? | YES | Fix strategy label override |
| 5 | Does `clg .session.ensure strategy::resume` force "resume" output label? | YES | Fix strategy label override |
| 6 | Does `clg .session.ensure` create the dir if absent (both strategies)? | YES | Fix directory creation |
| 7 | Does `ctest3` pass with zero warnings? | YES | Fix warnings/failures |
| 8 | Are new commands documented in `docs/cli/commands.md`? | YES | Update docs |

## Validation Procedure

### Measurements

| # | Metric | Before | After | Command |
|---|--------|--------|-------|---------|
| M1 | Commands registered in `claude_storage` | N (current count) | N+4 | `clg .help \| grep -c '^\.'` |
| M2 | `ctest3` result | passing | passing | `ctest3` |

### Anti-faking Checks

- AF1: Run `clg .path` from a real project dir and verify the output path exists under `~/.claude/projects/` — confirms the handler uses real path encoding, not a stub return
- AF2: Run `clg .session.ensure path::/tmp/af2_test topic::smoke` twice; both runs must exit 0 and return the same path — confirms idempotent directory creation
- AF3: Run `clg .session.ensure path::/tmp/af2_test topic::smoke strategy::fresh` with pre-existing history; line 2 must be "fresh" — confirms strategy label override works

## Outcomes

- 4 new commands implemented: `.path`, `.exists`, `.session.dir`, `.session.ensure`
- 2 new parameters: `topic::` (TopicName), `strategy::` (StrategyType)
- 28 integration tests added to `tests/session_path_command_test.rs` — all passing
- YAML definitions added for all 4 commands in `unilang.commands.yaml`
- PHF map updated in `src/cli_main.rs` (13 commands total)
- `ctest3` passes: 241/241 tests, 3 doc tests, 0 clippy warnings
- `docs/cli/readme.md` Implementation Status updated to 100% (13/13)
- `spec.md` updated: 4 commands marked ✅ implemented

# Task 047 — Implement `.sessions` command

**Status:** ✅ Complete
**Category:** feature
**Created:** 2026-03-28

## Goal

Implement the `.sessions` CLI command for `claude_storage` / `cls`.

The command provides a **session-first** listing with scope control mirroring
`kbase`'s `scope::` parameter semantics. Where `.list` is project-first (shows
projects, optionally expanding sessions), `.sessions` is session-first: returns
all sessions matching the scope without requiring the user to think in terms of
projects first.

## Demand Origin

Spec-documented as "planned" in:
- `docs/cli/commands.md` § Command 8 `.sessions`
- `docs/cli/types.md` § Type 7 `ScopeValue`
- `docs/cli/testing/param/scope.md` (8 edge-case test scenarios EC-1..EC-8)

No task existed for implementation; gap identified 2026-03-28.

## Scope

`scope::` controls which projects' sessions are included:

| Value | Behavior |
|-------|----------|
| `local` | Current project only (default) |
| `relevant` | Walk ancestor chain from cwd → `/`, collect from every matching project |
| `under` | All projects whose path starts with `path::` |
| `global` | All projects in storage |

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `scope::` | enum | `local` | Discovery scope |
| `path::` | StoragePath | cwd | Base path for scope resolution |
| `session::` | SessionFilter | — | Filter sessions by ID substring |
| `agent::` | Boolean | — | Session type filter (0=main, 1=agent) |
| `min_entries::` | EntryCount | — | Minimum entry count threshold |
| `verbosity::` | VerbosityLevel | 1 | Output detail level (0-5) |

## Done When

- [x] `.sessions` present in `unilang.commands.yaml`
- [x] `sessions_routine` implemented in `src/cli/mod.rs`
- [x] `sessions_routine` registered in `src/main.rs`
- [x] `tests/sessions_command_test.rs` passes (EC-1..EC-8 + behavioural tests)
- [x] `spec.md` updated: command added (9 commands total)
- [x] `w3 .test l::3` passes clean (174/174 tests, 0 clippy warnings)

## Outcomes

`sessions_routine` added at `src/cli/mod.rs:1870` — session-first view complementing
`.list`'s project-first view. All four scope values (local/relevant/under/global) implemented
using `ProjectId::Path` `starts_with` / equality comparisons. Six parameters registered in
`unilang.commands.yaml` and validated inline with early-return error paths.

16 tests in `tests/sessions_command_test.rs`: EC-1..EC-8 verify scope parameter acceptance
(each scope value accepted; invalid value rejected); 4 behavioural tests verify correct
project selection per scope using synthetic TempDir fixtures; 4 validation tests cover
invalid `scope::`, `verbosity::`, `min_entries::`, and nonexistent `path::`.

Plan 002 executed atomically: 9 existing doc files promoted from "planned" to stable + 1
new file `docs/cli/testing/command/sessions.md` (IT-1..IT-8). All stale counters corrected:
8→9 commands in `spec.md`, 155→174 tests in `tests/readme.md`.

**Final state:** 174/174 tests pass, 0 clippy warnings, 9 CLI commands stable.

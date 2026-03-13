# TSK-074 — Add typed builder methods for session management parameters

## Status

✅ (Completed)

## Metadata

- **Value:** 7
- **Easiness:** 7
- **Priority:** 2
- **Safety:** 8
- **Advisability:** 784

## Goal

Add typed `with_*` builder methods for `resume`, `session_id`, `fork_session`, `no_session_persistence`, and `from_pr`.

**MOST criteria:**
- **Motivated:** Session continuity is a core automation pattern — resuming a conversation or pinning a session ID requires hand-crafted raw args today.
- **Observable:** Five new typed methods; `describe()` reflects correct flag syntax.
- **Scoped:** `src/command.rs` only.
- **Testable:** `ctest3` green; tests verify each flag appears correctly.

## Description

Session management parameters control conversation continuity across invocations. `--resume [id]` resumes the most recent conversation or a specific session by ID. `--session-id <uuid>` pins the UUID for the session. `--fork-session` creates a new session ID on resume. `--no-session-persistence` disables save-to-disk. `--from-pr [value]` resumes a session linked to a PR.

`resume` takes an optional ID — the builder should support both `with_resume(None)` (most-recent) and `with_resume(Some("uuid"))` (specific). `session_id` and `from_pr` take required string values. `fork_session` and `no_session_persistence` are boolean flags.

## In Scope

- `with_resume(id: Option<&str>)` — adds `-r` alone or `-r <id>` with value
- `with_session_id<S: Into<String>>(id: S)` — adds `--session-id <uuid>`
- `with_fork_session(bool)` — adds `--fork-session` when true
- `with_no_session_persistence(bool)` — adds `--no-session-persistence` when true
- `with_from_pr<S: Into<String>>(value: S)` — adds `--from-pr <value>`
- Integration tests for all five methods
- Update `docs/claude_params/readme.md` Builder column for all five params

## Out of Scope

- UUID format validation for `session_id`
- Session file existence checking

## Requirements

- Follow `code_design.rulebook.md` — TDD red-green-refactor
- Follow `codebase_hygiene.rulebook.md` — no mocking
- Follow `test_organization.rulebook.md` — tests in `tests/` only
- Follow `code_style.rulebook.md` — 2-space indent, custom codestyle

## Acceptance Criteria

- `with_resume(None)` adds `-r` with no value
- `with_resume(Some("abc-123"))` adds `-r abc-123`
- `with_session_id("uuid-here")` adds `--session-id uuid-here`
- `with_fork_session(true)` adds `--fork-session`; `with_fork_session(false)` adds nothing
- `with_no_session_persistence(true)` adds `--no-session-persistence`
- `with_from_pr("42")` adds `--from-pr 42`
- `ctest3` green; zero warnings

## Work Procedure

1. Read applicable rulebooks via `kbase .rulebooks`
2. Write failing tests for all five methods (RED)
3. Implement all five `with_*` methods in `src/command.rs`
4. All tests green (GREEN)
5. Refactor if needed; verify `ctest3`
6. Update `docs/claude_params/readme.md` Builder column
7. Update task status → ✅

## Validation Checklist

| # | Question | Desirable | Fail Action |
|---|----------|-----------|-------------|
| 1 | Does `with_resume(None)` add `-r` without a value argument? | YES | Fix option handling |
| 2 | Does `with_resume(Some("id"))` add `-r id`? | YES | Fix method |
| 3 | Does `with_fork_session(false)` add nothing? | YES | Fix boolean guard |
| 4 | Does `ctest3` pass? | YES | Fix failures |

## Validation Procedure

### Measurements

| # | Metric | Before | After | Command |
|---|--------|--------|-------|---------|
| M1 | Public `with_*` methods on `ClaudeCommand` | 33 | 38 | `grep -c 'pub fn with_' src/command.rs` |
| M2 | `ctest3` result | passing | passing | `ctest3` |

### Anti-faking Checks

- AF1: Build with `with_resume(None)` and assert `describe()` contains `-r` but NOT `-r ` followed by a UUID — confirms optional-value handling
- AF2: Build with `with_fork_session(true)` and `with_no_session_persistence(true)` simultaneously; assert both flags appear in describe()

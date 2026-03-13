# Move SessionManager from claude_session to claude_runner_core

## Status

✅ **Complete** — 2026-03-21

## Goal

Move `claude_session/src/session.rs` (`SessionManager`, `Strategy`) to
`claude_runner_core/src/session_dir.rs`. `SessionManager` manages LOCAL
invocation directories (`-{topic}/`), not `~/.claude/projects/` — it is
execution preparation logic, not credential management.

## In Scope

- Create `claude_runner_core/src/session_dir.rs` with `SessionManager` and `Strategy`
- Create `claude_runner_core/tests/session_dir_tests.rs` (18 tests, from session_tests.rs
  and relevant corner_case_tests.rs SF-1..SF-5, SM-4)
- Update `claude_runner_core/src/lib.rs` to export new module
- Update `claude_runner_core/Cargo.toml` (add `tempfile` dev-dep)
- Delete `claude_session/src/session.rs`
- Remove `session` from `claude_session/src/lib.rs`, `spec.md`, `readme.md`
- Delete `claude_session/tests/session_tests.rs` and `corner_case_tests.rs` (content moved)

## Out of Scope

- Renaming `SessionManager` → `InvocationDirManager` (deferred, separate task if needed)
- Any changes to `ClaudeCommand` or execution logic

## Key Distinction

`SessionManager::ensure_session()` creates/deletes LOCAL `-{topic}/` directories.
It does NOT touch `~/.claude/projects/`. The `Fresh` strategy deletes only the
local invocation directory, not Claude's stored conversation history.

## Completed Work

- `claude_runner_core/src/session_dir.rs` — created (SessionManager + Strategy)
- `claude_runner_core/tests/session_dir_tests.rs` — created (18 tests)
- `claude_runner_core/src/lib.rs` — added `pub mod session_dir` and exports
- `claude_runner_core/Cargo.toml` — added `tempfile` dev-dep
- `claude_session/src/session.rs` — deleted
- `claude_session/src/lib.rs` — removed session module and re-exports
- `claude_session/tests/session_tests.rs` — deleted (content migrated)
- `claude_session/tests/corner_case_tests.rs` — deleted (SF/SM content migrated)

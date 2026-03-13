# Move Continuation Detection from claude_session to claude_storage_core

## Status

✅ **Complete** — 2026-03-21

## Goal

Move `claude_session/src/detection.rs` logic (`check_session_exists`,
`get_claude_storage_path`) to `claude_storage_core/src/continuation.rs`
with corrected v1 path encoding. Fixes responsibility overlap: session file
access belongs in `claude_storage_core`, not `claude_session`.

## In Scope

- Create `claude_storage_core/src/continuation.rs` with `check_continuation()`
  and `to_storage_path_for()`
- Create `claude_storage_core/tests/continuation_tests.rs` (8 tests)
- Update `claude_storage_core/src/lib.rs` to export new module
- Update `claude_storage_core/spec.md` (add continuation detection to In Scope,
  fix Out of Scope: `claude_session` → `claude_runner_core`)
- Delete `claude_session/src/detection.rs`
- Remove `detection` from `claude_session/src/lib.rs`, `spec.md`, `readme.md`
- Delete `claude_session/tests/detection_tests.rs` (content moved)

## Out of Scope

- Implementing `.session` command in `claude_storage` CLI → task-035
- Moving `SessionManager` → task-034

## Key Fix

`detection.rs` escaped `/_.@#%& ` → `-` (overly broad). `claude_storage_core`'s
`encode_path()` only escapes `/` and `_` (correct v1 encoding). Moving to
`claude_storage_core` and using `encode_path()` fixes the path encoding bug.

## Completed Work

- `claude_storage_core/src/continuation.rs` — created
- `claude_storage_core/tests/continuation_tests.rs` — created (8 tests)
- `claude_storage_core/src/lib.rs` — added `pub mod continuation` and exports
- `claude_storage_core/spec.md` — updated In Scope and Out of Scope
- `claude_session/src/detection.rs` — deleted
- `claude_session/src/lib.rs` — removed detection module and re-exports
- `claude_session/tests/detection_tests.rs` — deleted (content migrated)

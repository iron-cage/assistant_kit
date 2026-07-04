# Algorithm: Path Encoding (Df())

### Scope

- **Purpose**: Document how `claude_runner` consumes the Df() path-encoding algorithm to compute session and memory storage paths.
- **Responsibility**: Identify claude_runner's consumption points for Df() and point to its canonical specification.
- **In Scope**: Consumption points (`scope_for()`, `to_storage_path_for()`), cross-crate reference to the canonical algorithm doc.
- **Out of Scope**: Encoding/decoding steps, character mapping, and edge cases — canonical specification lives in `claude_storage_core` (see Algorithm Ownership below); git root detection (→ `002_git_root_detection.md`); session file selection (→ `003_session_file_selection.md`).

### Algorithm Ownership

The Df() path-encoding algorithm (`encode_path()` / `decode_path()`) is implemented and canonically documented in `claude_storage_core`, not in `claude_runner`. `claude_runner` depends on `claude_storage_core::scope::scope_for()` and `claude_storage_core::continuation::to_storage_path_for()`, both of which call `encode_path()` internally.

| Type | File | Relationship |
|------|------|--------------|
| canonical doc | [`../../../claude_storage_core/docs/algorithm/001_path_encoding.md`](../../../claude_storage_core/docs/algorithm/001_path_encoding.md) | Full algorithm specification: encoding steps, decoding heuristic, edge cases, known limitations |
| source | `claude_storage_core/src/scope.rs` — `scope_for()` | Computes all 6 `CLAUDE_*` variables using Df() |
| source | `claude_storage_core/src/continuation.rs` — `to_storage_path_for()` | Applies Df() to compute `~/.claude/projects/{encoded}/` |

### Related Docs

| File | Relationship |
|------|--------------|
| [`002_git_root_detection.md`](002_git_root_detection.md) | Git root detection — uses Df() on the resolved git root for memory path |
| [`003_session_file_selection.md`](003_session_file_selection.md) | Session selection — operates on the directory computed by Df() |
| [`../variable/001_claude_home.md`](../variable/001_claude_home.md) | CLAUDE_HOME — base used before Df() is applied |
| [`../variable/002_claude_projects_dir.md`](../variable/002_claude_projects_dir.md) | CLAUDE_PROJECTS_DIR — Df() output appended to this |
| [`../variable/003_claude_session_dir.md`](../variable/003_claude_session_dir.md) | CLAUDE_SESSION_DIR — full session path using Df(target_dir) |
| [`../variable/004_claude_memory_dir.md`](../variable/004_claude_memory_dir.md) | CLAUDE_MEMORY_DIR — full memory path using Df(git_root(target_dir)) |
| [`../feature/005_session_path_resolution.md`](../feature/005_session_path_resolution.md) | Feature hub: `scope_for()` and session cross-loading |

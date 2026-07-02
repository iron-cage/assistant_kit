# algorithm/

Path computation algorithm specifications for the `claude_runner` crate.

### Scope

- **Purpose:** Document the non-trivial algorithms used for Claude Code path derivation.
- **Responsibility:** Index all algorithm doc instances — one per distinct algorithm.
- **In Scope:** Path encoding (Df()); git root upward walk; mtime-based session file selection.
- **Out of Scope:** Per-variable reference details (see `variable/`); feature behavior contracts (see `feature/`); CLI command specs (see `cli/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `001_path_encoding.md` | Df() path encoding algorithm — converts absolute path to session/memory directory segment |
| `002_git_root_detection.md` | Git root upward-walk algorithm — resolves the memory key from a working directory |
| `003_session_file_selection.md` | mtime-based session file selection — replicates `claude -c` active-session choice |

### Overview Table

| ID | File | Name | Status |
|----|------|------|--------|
| 001 | [001_path_encoding.md](001_path_encoding.md) | Path Encoding | ✅ |
| 002 | [002_git_root_detection.md](002_git_root_detection.md) | Git Root Detection | ✅ |
| 003 | [003_session_file_selection.md](003_session_file_selection.md) | Session File Selection | ✅ |

# Variable Doc Entity

Output variable definitions for the six CLAUDE_* paths computed by `scope_for()`.

### Scope

- **Purpose:** Document each CLAUDE_* variable — its value format, derivation rule, and env-var overrides.
- **Responsibility:** Index all variable doc instances — one per output variable.
- **In Scope:** CLAUDE_HOME, CLAUDE_PROJECTS_DIR, CLAUDE_SESSION_DIR, CLAUDE_MEMORY_DIR, CLAUDE_MEMORY_FILE, CLAUDE_SESSION_FILE.
- **Out of Scope:** Algorithm internals (see `algorithm/`); feature behavior contracts (see `feature/`); CLI command specs (see `cli/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `001_claude_home.md` | CLAUDE_HOME — base Claude config directory |
| `002_claude_projects_dir.md` | CLAUDE_PROJECTS_DIR — session and memory root under CLAUDE_HOME |
| `003_claude_session_dir.md` | CLAUDE_SESSION_DIR — per-working-directory session storage |
| `004_claude_memory_dir.md` | CLAUDE_MEMORY_DIR — per-git-root memory directory |
| `005_claude_memory_file.md` | CLAUDE_MEMORY_FILE — canonical MEMORY.md path |
| `006_claude_session_file.md` | CLAUDE_SESSION_FILE — most recently active session file path |

### Overview Table

| ID | File | Name | Override |
|----|------|------|----------|
| 001 | [001_claude_home.md](001_claude_home.md) | CLAUDE_HOME | `CLAUDE_HOME` env var |
| 002 | [002_claude_projects_dir.md](002_claude_projects_dir.md) | CLAUDE_PROJECTS_DIR | — |
| 003 | [003_claude_session_dir.md](003_claude_session_dir.md) | CLAUDE_SESSION_DIR | — |
| 004 | [004_claude_memory_dir.md](004_claude_memory_dir.md) | CLAUDE_MEMORY_DIR | `CLAUDE_COWORK_MEMORY_PATH_OVERRIDE` env var |
| 005 | [005_claude_memory_file.md](005_claude_memory_file.md) | CLAUDE_MEMORY_FILE | (follows CLAUDE_MEMORY_DIR) |
| 006 | [006_claude_session_file.md](006_claude_session_file.md) | CLAUDE_SESSION_FILE | — |

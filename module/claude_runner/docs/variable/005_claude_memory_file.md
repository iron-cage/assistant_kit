# Variable: CLAUDE_MEMORY_FILE

### Scope

- **Purpose**: Document the canonical MEMORY.md path — the agent memory file Claude Code reads and updates during a session.
- **Responsibility**: Define the value format, derivation rule, and examples for `CLAUDE_MEMORY_FILE`.
- **In Scope**: CLAUDE_MEMORY_FILE derivation from CLAUDE_MEMORY_DIR.
- **Out of Scope**: CLAUDE_MEMORY_DIR derivation (→ `004_claude_memory_dir.md`); MEMORY.md content or format; override via `CLAUDE_COWORK_MEMORY_PATH_OVERRIDE` (→ `004_claude_memory_dir.md`).

### Value Format

Absolute filesystem path to a Markdown file. Always ends with `/MEMORY.md`.

**Examples:**
- `/home/alice/.claude/projects/-home-alice-project/memory/MEMORY.md`
- `/opt/shared-memory/MEMORY.md` (when `CLAUDE_COWORK_MEMORY_PATH_OVERRIDE` is set)

### Derivation

```
CLAUDE_MEMORY_FILE = $CLAUDE_MEMORY_DIR + "MEMORY.md"
```

`CLAUDE_MEMORY_FILE` is a pure suffix extension of `CLAUDE_MEMORY_DIR`. It inherits all of `CLAUDE_MEMORY_DIR`'s derivation rules and override behavior.

### Override

| Mechanism | Value |
|-----------|-------|
| Env var | none directly — override `CLAUDE_COWORK_MEMORY_PATH_OVERRIDE` to change `CLAUDE_MEMORY_DIR`, which changes `CLAUDE_MEMORY_FILE` |

### Examples

| CLAUDE_MEMORY_DIR | CLAUDE_MEMORY_FILE |
|-------------------|--------------------|
| `/home/alice/.claude/projects/-home-alice-project/memory/` | `/home/alice/.claude/projects/-home-alice-project/memory/MEMORY.md` |
| `/opt/shared-memory/` | `/opt/shared-memory/MEMORY.md` |
| `/root/.claude/projects/-tmp-scratch/memory/` | `/root/.claude/projects/-tmp-scratch/memory/MEMORY.md` |

### Related Docs

| File | Relationship |
|------|--------------|
| [`004_claude_memory_dir.md`](004_claude_memory_dir.md) | CLAUDE_MEMORY_DIR — prefix for CLAUDE_MEMORY_FILE |
| [`../algorithm/002_git_root_detection.md`](../algorithm/002_git_root_detection.md) | Git root detection — determines which project tree MEMORY.md lives in |
| [`../feature/005_session_path_resolution.md`](../feature/005_session_path_resolution.md) | Feature hub: `scope_for()` and session cross-loading |

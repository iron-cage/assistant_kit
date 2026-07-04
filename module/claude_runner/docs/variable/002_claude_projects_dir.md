# Variable: CLAUDE_PROJECTS_DIR

### Scope

- **Purpose**: Document the session and memory root directory — the parent of all per-directory Claude Code storage trees.
- **Responsibility**: Define the value format, derivation rule, and examples for `CLAUDE_PROJECTS_DIR`.
- **In Scope**: CLAUDE_PROJECTS_DIR derivation from CLAUDE_HOME, relation to CLAUDE_SESSION_DIR and CLAUDE_MEMORY_DIR.
- **Out of Scope**: CLAUDE_HOME override (→ `001_claude_home.md`); downstream variables (→ `003_claude_session_dir.md` through `006_claude_session_file.md`); algorithm details (→ `../algorithm/`).

### Value Format

Absolute filesystem path to a directory. Always ends with `/`.

**Examples:** `/home/alice/.claude/projects/`, `/opt/claude-config/projects/`

### Derivation

```
CLAUDE_PROJECTS_DIR = $CLAUDE_HOME/projects/
```

Appends `projects/` to `CLAUDE_HOME`. No environment variable override exists for this variable — it always follows `CLAUDE_HOME`.

### Override

| Mechanism | Value |
|-----------|-------|
| Env var | none — override `CLAUDE_HOME` to relocate the entire tree |

To change `CLAUDE_PROJECTS_DIR`, set `CLAUDE_HOME` to a different path.

### Examples

| CLAUDE_HOME | CLAUDE_PROJECTS_DIR |
|-------------|---------------------|
| `/home/alice/.claude` | `/home/alice/.claude/projects/` |
| `/root/.claude` | `/root/.claude/projects/` |
| `/opt/claude-config` | `/opt/claude-config/projects/` |

**Directory structure under CLAUDE_PROJECTS_DIR:**

```
~/.claude/projects/
  -home-alice-project/          ← CLAUDE_SESSION_DIR for /home/alice/project
    9a3f8a12-cdef-...jsonl      ← session file
    memory/                     ← CLAUDE_MEMORY_DIR
      MEMORY.md                 ← CLAUDE_MEMORY_FILE
  -home-alice-other-project/    ← CLAUDE_SESSION_DIR for /home/alice/other-project
    ...
```

### Related Docs

| File | Relationship |
|------|--------------|
| [`001_claude_home.md`](001_claude_home.md) | CLAUDE_HOME — CLAUDE_PROJECTS_DIR = CLAUDE_HOME + "/projects/" |
| [`003_claude_session_dir.md`](003_claude_session_dir.md) | CLAUDE_SESSION_DIR = CLAUDE_PROJECTS_DIR + Df(target_dir) |
| [`004_claude_memory_dir.md`](004_claude_memory_dir.md) | CLAUDE_MEMORY_DIR = CLAUDE_PROJECTS_DIR + Df(git_root(target_dir)) + "/memory/" |
| [`../algorithm/001_path_encoding.md`](../algorithm/001_path_encoding.md) | Df() — applied after CLAUDE_PROJECTS_DIR to produce session/memory paths |
| [`../feature/005_session_path_resolution.md`](../feature/005_session_path_resolution.md) | Feature hub: `scope_for()` and session cross-loading |

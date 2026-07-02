# Variable: CLAUDE_MEMORY_DIR

### Scope

- **Purpose**: Document the per-git-root memory directory — the location where Claude Code stores the `MEMORY.md` agent memory file for a project.
- **Responsibility**: Define the value format, derivation rule (Df() applied to git root), env var override, and examples for `CLAUDE_MEMORY_DIR`.
- **In Scope**: CLAUDE_MEMORY_DIR derivation using git root detection, `CLAUDE_COWORK_MEMORY_PATH_OVERRIDE` env var.
- **Out of Scope**: Session storage (→ `003_claude_session_dir.md`); CLAUDE_HOME/CLAUDE_PROJECTS_DIR (→ `001_claude_home.md`, `002_claude_projects_dir.md`); MEMORY.md content; git root detection algorithm details (→ `../algorithm/002_git_root_detection.md`).

### Value Format

Absolute filesystem path to a directory. Always ends with `/memory/`.

**Examples:**
- `/home/alice/.claude/projects/-home-alice-project/memory/`
- `/opt/shared-memory/` (when override is set)

### Derivation

```
CLAUDE_MEMORY_DIR = (
  $CLAUDE_COWORK_MEMORY_PATH_OVERRIDE   if set and non-empty
  | $CLAUDE_PROJECTS_DIR + Df(git_root(target_dir)) + "/memory/"
)
```

**Normal case (no override):**
1. Run git root detection on `target_dir` (→ `../algorithm/002_git_root_detection.md`).
2. Apply Df() to the git root.
3. Append `/memory/`.

**Step-by-step for `/home/alice/project/src` with git root at `/home/alice/project`:**
```
target_dir             = /home/alice/project/src
git_root(target_dir)   = /home/alice/project
Df(git_root)           = -home-alice-project
CLAUDE_MEMORY_DIR      = /home/alice/.claude/projects/-home-alice-project/memory/
```

**Why git root (not target_dir)?** All subdirectories of a project share the same MEMORY.md. Running Claude from `/project/src` and from `/project` should use the same memory file. Anchoring to the git root achieves this.

### Override

| Mechanism | Value |
|-----------|-------|
| Env var | `CLAUDE_COWORK_MEMORY_PATH_OVERRIDE` — set to any absolute path; bypasses all derivation |

The override is intended for co-working scenarios where multiple team members share a single memory store at a fixed path.

```sh
# All invocations use the same shared memory
CLAUDE_COWORK_MEMORY_PATH_OVERRIDE=/shared/claude/memory clr scope
```

### Examples

| target_dir | git_root | CLAUDE_MEMORY_DIR |
|------------|----------|-------------------|
| `/home/alice/project/src` | `/home/alice/project` | `/home/alice/.claude/projects/-home-alice-project/memory/` |
| `/home/alice/project` | `/home/alice/project` | `/home/alice/.claude/projects/-home-alice-project/memory/` |
| `/tmp/scratch` (no git) | `/tmp/scratch` (fallback) | `/home/alice/.claude/projects/-tmp-scratch/memory/` |
| any dir | any | `CLAUDE_COWORK_MEMORY_PATH_OVERRIDE` value (override active) |

### Related Docs

| File | Relationship |
|------|--------------|
| [`002_claude_projects_dir.md`](002_claude_projects_dir.md) | CLAUDE_PROJECTS_DIR — prefix before Df(git_root) |
| [`005_claude_memory_file.md`](005_claude_memory_file.md) | CLAUDE_MEMORY_FILE = CLAUDE_MEMORY_DIR + "MEMORY.md" |
| [`../algorithm/001_path_encoding.md`](../algorithm/001_path_encoding.md) | Df() — applied to git_root(target_dir) |
| [`../algorithm/002_git_root_detection.md`](../algorithm/002_git_root_detection.md) | Git root detection — resolves the memory anchor |
| [`../feature/005_session_path_resolution.md`](../feature/005_session_path_resolution.md) | Feature hub: `scope_for()` and session cross-loading |

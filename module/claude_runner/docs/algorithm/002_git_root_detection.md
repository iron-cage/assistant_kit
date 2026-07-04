# Algorithm: Git Root Detection

### Scope

- **Purpose**: Resolve the memory anchor for a working directory by locating the nearest `.git`-containing ancestor directory.
- **Responsibility**: Specify the upward-walk algorithm, `.git` detection conditions, filesystem root handling, and the fallback behavior when no git repository is found.
- **In Scope**: Upward walk steps, `.git` detection (file and directory), filesystem root sentinel, fallback to `start_dir`.
- **Out of Scope**: Path encoding applied after git root detection (→ `001_path_encoding.md`); CLAUDE_MEMORY_DIR computation (→ `../variable/004_claude_memory_dir.md`); session file selection (→ `003_session_file_selection.md`).

### Algorithm

Given an input directory `start_dir`:

1. Set `current` = `start_dir`.
2. Check if `current/.git` exists — as either a **directory** (normal git repository) or a **file** (git worktree pointer).
3. If `.git` exists: return `current` as the git root.
4. Set `current` = parent of `current`.
5. If `current` has no parent (filesystem root reached): return `start_dir` as fallback.
6. Go to step 2.

**Fallback semantics:** When no `.git` is found all the way to the filesystem root, the algorithm returns `start_dir` unchanged. This ensures `CLAUDE_MEMORY_DIR` is always well-defined — it falls back to the project-local memory path rather than failing.

### Examples

| `start_dir` | `.git` Location | Result |
|-------------|-----------------|--------|
| `/home/alice/project/src` | `/home/alice/project/.git` | `/home/alice/project` |
| `/home/alice/project` | `/home/alice/project/.git` | `/home/alice/project` |
| `/home/alice/project` | none (no git repo) | `/home/alice/project` (fallback) |
| `/home/alice/worktree` | `/home/alice/worktree/.git` (file) | `/home/alice/worktree` |
| `/tmp/scratch` | none | `/tmp/scratch` (fallback) |

**Effect on CLAUDE_MEMORY_DIR:**

```
# With git repo rooted at /home/alice/project
start_dir = /home/alice/project/src
git_root  = /home/alice/project
CLAUDE_MEMORY_DIR = ~/.claude/projects/Df(/home/alice/project)/memory/
                  = ~/.claude/projects/-home-alice-project/memory/

# Without git repo (fallback)
start_dir = /tmp/scratch
git_root  = /tmp/scratch  (fallback)
CLAUDE_MEMORY_DIR = ~/.claude/projects/-tmp-scratch/memory/
```

### Edge Cases

| Scenario | Behavior |
|----------|----------|
| `.git` is a file (git worktree) | Treated as present — file existence satisfies detection |
| `.git` is a directory (normal repo) | Standard case |
| No `.git` anywhere up to filesystem root | Fallback: return `start_dir` |
| `start_dir` IS the git root | Returns immediately at step 2 |
| Filesystem root `/` as `start_dir` | Returns `/` (no parent) |

### Implementation

| Location | Symbol | Role |
|----------|--------|------|
| `claude_storage_core/src/scope.rs` | `git_root_for(dir: &Path) -> PathBuf` | Upward-walk git root detector |
| `claude_storage_core/src/scope.rs` | `scope_for(dir: &Path) -> ClaudeScope` | Calls `git_root_for()` to compute `claude_memory_dir` |

### Related Docs

| File | Relationship |
|------|--------------|
| [`001_path_encoding.md`](001_path_encoding.md) | Df() — applied to the resolved git root to produce the memory storage segment |
| [`003_session_file_selection.md`](003_session_file_selection.md) | Session file selection — parallel algorithm for `CLAUDE_SESSION_FILE` |
| [`../variable/004_claude_memory_dir.md`](../variable/004_claude_memory_dir.md) | CLAUDE_MEMORY_DIR — output of Df(git_root(dir)) + `/memory/` |
| [`../variable/005_claude_memory_file.md`](../variable/005_claude_memory_file.md) | CLAUDE_MEMORY_FILE — `CLAUDE_MEMORY_DIR/MEMORY.md` |
| [`../feature/005_session_path_resolution.md`](../feature/005_session_path_resolution.md) | Feature hub: `scope_for()` and session cross-loading |

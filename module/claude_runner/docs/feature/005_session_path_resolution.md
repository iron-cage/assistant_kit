# Feature: Session Path Resolution

### Scope

- **Purpose**: Define the `scope_for()` function contract — how all 6 `CLAUDE_*` path variables are computed for any target directory — and the session cross-loading behaviors enabled by it.
- **Responsibility**: Specify what `scope_for(dir)` produces, the `clr scope` inspection command, and the `--session-from` / `--to` parameters for session cross-loading.
- **In Scope**: `scope_for()` function contract, `ClaudeScope` struct fields, `git_root_for()` helper, `clr scope` command, `--session-from` parameter, `--to` alias for `--dir`, CLAUDE_HOME env var fix in `claude_storage_core`.
- **Out of Scope**: Algorithm internals (→ `../algorithm/`); per-variable format specs (→ `../variable/`); session cross-loading isolation constraint (→ `../invariant/011_session_source_isolation.md`); CLI parameter syntax (→ `../cli/param/076_session_from.md`, `../cli/command/09_scope.md`).

### Feature Description

**`scope_for(dir: &Path) -> ClaudeScope`** computes all 6 `CLAUDE_*` path variables for a given directory in a single call. It is the canonical entry point for any code that needs to know where Claude Code stores sessions or memory for a directory.

**`ClaudeScope` struct** (new type in `claude_storage_core::scope`):

| Field | Type | Value |
|-------|------|-------|
| `claude_home` | `PathBuf` | `${CLAUDE_HOME:-$HOME/.claude}` |
| `claude_projects_dir` | `PathBuf` | `claude_home/projects/` |
| `claude_session_dir` | `PathBuf` | `claude_projects_dir/Df(dir)` |
| `claude_memory_dir` | `PathBuf` | `claude_projects_dir/Df(git_root(dir))/memory/` (or override) |
| `claude_memory_file` | `PathBuf` | `claude_memory_dir/MEMORY.md` |
| `claude_session_file` | `Option<PathBuf>` | highest-mtime `.jsonl` in `claude_session_dir`; `None` if absent |

**`git_root_for(dir: &Path) -> PathBuf`** — pure helper that walks up from `dir` looking for `.git`; falls back to `dir` if none found. Used by `scope_for()` to anchor `claude_memory_dir` to the project root.

**CLAUDE_HOME env var handling:** Both `to_storage_path_for()` (`continuation.rs`) and `scope_for()` (`scope.rs`) check `$CLAUDE_HOME` first, falling back to `$HOME/.claude`. This ensures `clr scope` and `--session-from` respect `CLAUDE_HOME` overrides.

### Session Cross-Loading

Session cross-loading lets Claude run in one directory while loading its initial session from another directory's session history. Two scenarios are supported:

**Scenario 1 — Clone Outward** ("run in B, use session from A")

```sh
clr --to /home/alice/project-b --session-from /home/alice/project-a "Continue this feature"
```

Claude runs in `/home/alice/project-b` but starts from the most recent session of `/home/alice/project-a`. This is useful when branching work to a new project directory.

**Scenario 2 — Inject Inward** ("run in A, use session from B")

```sh
clr --session-from /home/alice/project-b "What did you do in B?"
```

Claude runs in CWD (or `--dir`) but loads the session from `/home/alice/project-b`. This is useful when you want to query or build on work done in another directory.

**Mechanics:**
- `--session-from <DIR>` computes `scope_for(DIR).claude_session_dir` and uses it to find the source session (via `most_recent_session_in_dir()`).
- The session UUID is injected via `-c <uuid>` to the claude subprocess.
- Claude then runs in the TARGET directory — future sessions are written to the TARGET's `CLAUDE_SESSION_DIR`, not the source's.
- This is one-time cross-loading: not persistent session mirroring.

**`--to` alias for `--dir`:** Enables the ergonomic pair `--to /b --session-from /a` while keeping `--dir` as the canonical parameter name.

### `clr scope` Command

Prints all 6 `CLAUDE_*` variables in `key=value` format suitable for shell eval or inspection:

```sh
$ clr scope --dir /home/alice/project
CLAUDE_HOME=/home/alice/.claude
CLAUDE_PROJECTS_DIR=/home/alice/.claude/projects/
CLAUDE_SESSION_DIR=/home/alice/.claude/projects/-home-alice-project
CLAUDE_MEMORY_DIR=/home/alice/.claude/projects/-home-alice-project/memory/
CLAUDE_MEMORY_FILE=/home/alice/.claude/projects/-home-alice-project/memory/MEMORY.md
CLAUDE_SESSION_FILE=/home/alice/.claude/projects/-home-alice-project/9a3f8a12-cdef-4567-8901-abcdef012345.jsonl
```

`CLAUDE_SESSION_FILE` is empty when no session history exists:

```sh
CLAUDE_SESSION_FILE=
```

### Acceptance Criteria

- **AC-1**: `scope_for(dir)` returns correct values for all 6 fields when `CLAUDE_HOME` is not set (uses `$HOME/.claude`).
- **AC-2**: `scope_for(dir)` respects `CLAUDE_HOME` env var override for `claude_home`, `claude_projects_dir`, `claude_session_dir`, `claude_memory_dir`, `claude_memory_file`.
- **AC-3**: `scope_for(dir)` respects `CLAUDE_COWORK_MEMORY_PATH_OVERRIDE` for `claude_memory_dir` and `claude_memory_file`.
- **AC-4**: `scope_for("/project/src")` with `.git` at `/project` returns `claude_memory_dir` rooted at `/project` (not `/project/src`).
- **AC-5**: `scope_for(dir)` returns `claude_session_file = None` when `CLAUDE_SESSION_DIR` is empty or missing.
- **AC-6**: `clr scope` prints all 6 `CLAUDE_*` variables for CWD (or `--dir`) in `key=value` format, one per line.
- **AC-7**: `clr run --session-from /src-dir "message"` causes Claude to resume the most recent session from `scope_for(/src-dir).claude_session_dir`, while running in CWD.
- **AC-8**: `clr run --to /target-dir --session-from /src-dir "message"` causes Claude to run in `/target-dir` with the session from `scope_for(/src-dir).claude_session_dir`.
- **AC-9**: `--to` is a usable alias for `--dir` in `run` and `ask`; behavior is identical.
- **AC-10**: `--session-from` is a higher-level companion to `--session-dir`; when both are given, `--session-dir` takes precedence (raw path beats computed).

### Related Docs

| File | Relationship |
|------|--------------|
| [`../algorithm/001_path_encoding.md`](../algorithm/001_path_encoding.md) | Df() — applied by scope_for() to compute session/memory paths |
| [`../algorithm/002_git_root_detection.md`](../algorithm/002_git_root_detection.md) | git_root_for() — used by scope_for() to anchor memory path |
| [`../algorithm/003_session_file_selection.md`](../algorithm/003_session_file_selection.md) | Session file selection — used by scope_for() to compute CLAUDE_SESSION_FILE |
| [`../variable/001_claude_home.md`](../variable/001_claude_home.md) | CLAUDE_HOME |
| [`../variable/002_claude_projects_dir.md`](../variable/002_claude_projects_dir.md) | CLAUDE_PROJECTS_DIR |
| [`../variable/003_claude_session_dir.md`](../variable/003_claude_session_dir.md) | CLAUDE_SESSION_DIR |
| [`../variable/004_claude_memory_dir.md`](../variable/004_claude_memory_dir.md) | CLAUDE_MEMORY_DIR |
| [`../variable/005_claude_memory_file.md`](../variable/005_claude_memory_file.md) | CLAUDE_MEMORY_FILE |
| [`../variable/006_claude_session_file.md`](../variable/006_claude_session_file.md) | CLAUDE_SESSION_FILE |
| [`../invariant/011_session_source_isolation.md`](../invariant/011_session_source_isolation.md) | Isolation constraint: session loaded from source, writes go to target |
| [`../cli/command/09_scope.md`](../cli/command/09_scope.md) | `clr scope` command reference |
| [`../cli/param/076_session_from.md`](../cli/param/076_session_from.md) | `--session-from` parameter reference |

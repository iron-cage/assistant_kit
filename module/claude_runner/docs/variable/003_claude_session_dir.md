# Variable: CLAUDE_SESSION_DIR

### Scope

- **Purpose**: Document the per-working-directory session storage path — the directory Claude Code uses to store conversation `.jsonl` files for a given execution directory.
- **Responsibility**: Define the value format, derivation rule (Df() applied to target dir), and examples for `CLAUDE_SESSION_DIR`.
- **In Scope**: CLAUDE_SESSION_DIR derivation, relationship to Df() encoding, `--session-from` override semantics.
- **Out of Scope**: CLAUDE_HOME/CLAUDE_PROJECTS_DIR (→ `001_claude_home.md`, `002_claude_projects_dir.md`); session file selection within the dir (→ `../algorithm/003_session_file_selection.md`); memory path (→ `004_claude_memory_dir.md`).

### Value Format

Absolute filesystem path to a directory. Never ends with `/` (the trailing slash is omitted for eval-safe output).

**Examples:**
- `/home/alice/.claude/projects/-home-alice-project`
- `/home/alice/.claude/projects/-home-alice-other-project`

### Derivation

```
CLAUDE_SESSION_DIR = $CLAUDE_PROJECTS_DIR + Df(target_dir)
```

Where:
- `target_dir` is the effective working directory (→ `--dir` if given, otherwise CWD)
- `Df()` is the path encoding algorithm (→ `../algorithm/001_path_encoding.md`)

**Step-by-step for `/home/alice/project`:**
```
target_dir          = /home/alice/project
Df(target_dir)      = -home-alice-project
CLAUDE_SESSION_DIR  = /home/alice/.claude/projects/-home-alice-project
```

### Override

| Mechanism | Value |
|-----------|-------|
| Env var | none |
| `--session-dir` param | Raw path override — bypasses Df() entirely; takes a verbatim directory path |
| `--session-from <DIR>` param | Computes `scope_for(DIR).claude_session_dir` and uses that for session lookup |

**`--session-from` vs `--session-dir`:**
- `--session-dir /path/to/dir` — takes the raw path verbatim as the session storage directory.
- `--session-from /home/alice/project` — computes `Df("/home/alice/project")` → uses `~/.claude/projects/-home-alice-project` as the source session dir. Higher-level ergonomic wrapper.

### Examples

| target_dir | CLAUDE_SESSION_DIR |
|------------|--------------------|
| `/home/alice/project` | `/home/alice/.claude/projects/-home-alice-project` |
| `/home/alice/other_project` | `/home/alice/.claude/projects/-home-alice-other-project` |
| `/tmp/scratch` | `/home/alice/.claude/projects/-tmp-scratch` |
| `/home/alice/my-app` | `/home/alice/.claude/projects/-home-alice-my-app` |

### Related Docs

| File | Relationship |
|------|--------------|
| [`002_claude_projects_dir.md`](002_claude_projects_dir.md) | CLAUDE_PROJECTS_DIR — prefix before Df(target_dir) |
| [`006_claude_session_file.md`](006_claude_session_file.md) | CLAUDE_SESSION_FILE — highest-mtime `.jsonl` inside CLAUDE_SESSION_DIR |
| [`../algorithm/001_path_encoding.md`](../algorithm/001_path_encoding.md) | Df() — converts target_dir to storage segment |
| [`../algorithm/003_session_file_selection.md`](../algorithm/003_session_file_selection.md) | Session file selection — scans CLAUDE_SESSION_DIR |
| [`../cli/param/010_session_dir.md`](../cli/param/010_session_dir.md) | `--session-dir` — raw path override |
| [`../cli/param/076_session_from.md`](../cli/param/076_session_from.md) | `--session-from` — computes CLAUDE_SESSION_DIR for source dir |
| [`../feature/005_session_path_resolution.md`](../feature/005_session_path_resolution.md) | Feature hub: `scope_for()` and session cross-loading |

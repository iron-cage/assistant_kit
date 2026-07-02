# User Story 028: Session Cross-Loading (Transplant)

### Scope

- **Persona**: Developer
- **Goal**: Run Claude in one project directory while resuming a session from a different project directory — either to branch work outward or to query/inject context inward.

### User Story

> As a developer working across multiple project directories,
> I want to run Claude in a target directory but continue a session from a source directory,
> so I can transplant context across projects without manually copying conversation history.

### Acceptance Criteria

- **AC-1 (Clone Outward):** `clr run --to /project-b --session-from /project-a "message"` causes Claude to run in `/project-b`, loading the most recent session from `/project-a`'s `CLAUDE_SESSION_DIR`. New conversation turns are written to `/project-b`'s session storage.
- **AC-2 (Inject Inward):** `clr run --session-from /project-b "message"` causes Claude to run in CWD, loading the most recent session from `/project-b`'s `CLAUDE_SESSION_DIR`.
- **AC-3 (No source history):** When `--session-from <DIR>` points to a directory with no qualifying session files, Claude starts a fresh session in the target directory (no error, no crash).
- **AC-4 (Alias `--from`):** `--from <DIR>` is accepted as a short alias for `--session-from <DIR>` with identical behavior.
- **AC-5 (Alias `--to`):** `--to <DIR>` is accepted as an alias for `--dir <DIR>` with identical behavior.
- **AC-6 (Precedence):** When both `--session-from` and `--session-dir` are given, `--session-dir` takes precedence.
- **AC-7 (Isolation):** The source directory's session files are never modified by the cross-loaded run.

### Primary Flags

| Flag | Role |
|------|------|
| `--session-from <DIR>` | Source directory for session lookup |
| `--from <DIR>` | Alias for `--session-from` |
| `--to <DIR>` | Alias for `--dir` (target directory where Claude runs) |
| `--dir <DIR>` | Target directory where Claude runs |

### Examples

```sh
# Clone outward: run in project-b, load session from project-a
clr "Continue this feature in the new project" \
  --to /home/alice/project-b \
  --session-from /home/alice/project-a

# Inject inward: run in CWD (project-a), query session from project-b
clr "What did you implement in project-b?" \
  --session-from /home/alice/project-b

# Short aliases
clr "Continue" --to /b --from /a
```

### Related Commands

| Command | Role |
|---------|------|
| `run` | Primary command for session cross-loading |
| `ask` | Also supports `--session-from`; identical to `run` |

### Related Doc Instances

| File | Relationship |
|------|--------------|
| [`../param/076_session_from.md`](../param/076_session_from.md) | `--session-from` parameter spec |
| [`../param/008_dir.md`](../param/008_dir.md) | `--dir` / `--to` parameter spec |
| [`../feature/005_session_path_resolution.md`](../feature/005_session_path_resolution.md) | Feature hub: `scope_for()` and cross-loading |
| [`../invariant/011_session_source_isolation.md`](../invariant/011_session_source_isolation.md) | Read/write isolation invariant |

### Related User Stories

| # | Title | Relationship |
|---|-------|--------------|
| 005 | [Project-specific Execution](005_project_specific_execution.md) | `--dir` for running in specific directory |
| 007 | [Fresh Session](007_fresh_session.md) | `--new-session` takes precedence over `--session-from` |
| 029 | [Scope Inspection](029_scope_inspection.md) | Use `clr scope` to verify source/target paths before cross-loading |

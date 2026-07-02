# Run Claude scoped to a project directory with isolated session storage

**Persona:** Developer working across multiple projects who needs Claude to operate in a specific project directory with isolated session state.
**Goal:** Run Claude scoped to a specific project directory and session storage location so context does not bleed between projects.
**Benefit:** Prevents cross-project context contamination and keeps Claude's working context relevant.
**Priority:** High

### Acceptance Criteria

- `--dir <path>` sets the subprocess working directory; Claude sees the given path as `cwd`
- `--session-dir <path>` stores and resumes session state from a project-specific location
- Both flags can be combined for full project isolation
- `--new-session` at the start of a new project task discards the previous session at that location

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`run`](../command/01_run.md) | Default command; `--dir` and `--session-dir` scope execution |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | `--dir` and `--session-dir` are runner control flags |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 6 | [`--interactive`](../param/006_interactive.md) | Continue interactively in the project directory |
| 7 | [`--new-session`](../param/007_new_session.md) | Discard prior session at that location |
| 8 | [`--dir`](../param/008_dir.md) | Set subprocess working directory |
| 10 | [`--session-dir`](../param/010_session_dir.md) | Set project-specific session storage path |

### Workflow Steps

1. `clr --dir /path/to/project "task"` — run Claude with the project directory as working directory
2. `clr --dir /path/to/project --session-dir /path/to/sessions "task"` — add project-specific session storage
3. `clr --dir /path/to/project --new-session "task"` — start a new task session in that project directory

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 22 | [022_session_isolation_subdir.md](022_session_isolation_subdir.md) | `--subdir` for task-level session isolation within the same project dir |

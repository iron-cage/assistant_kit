# User Story :: 005. Project-specific Execution

### Persona

Developer working across multiple projects who needs Claude to operate in a specific project directory with isolated session state.

### Goal

Run Claude scoped to a specific project directory and session storage location so context does not bleed between projects.

### Acceptance Criteria

- `--dir <path>` sets the subprocess working directory; Claude sees the given path as `cwd`
- `--session-dir <path>` stores and resumes session state from a project-specific location
- Both flags can be combined for full project isolation
- `--new-session` at the start of a new project task discards the previous session at that location

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [`run`](../001_command.md#command--1-run) | Both `--dir` and `--session-dir` apply to `run` |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`--dir`](../param/008_dir.md) | Set subprocess working directory |
| 2 | [`--session-dir`](../param/010_session_dir.md) | Set project-specific session storage path |
| 3 | [`--new-session`](../param/007_new_session.md) | Discard prior session at that location |
| 4 | [`--interactive`](../param/006_interactive.md) | Continue interactively in the project directory |

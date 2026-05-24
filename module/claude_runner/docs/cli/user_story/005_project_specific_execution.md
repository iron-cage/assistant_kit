# CLI User Story: Project-specific Execution

### Scope

- **Purpose**: Document project-scoped execution using --dir and --session-dir for isolation.
- **Responsibility**: Define acceptance criteria for directing Claude to a specific directory with isolated session state.
- **In Scope**: --dir subprocess cwd, --session-dir session storage, combined usage, --new-session at project start.
- **Out of Scope**: Credential isolation (→ 010_credential_isolated_execution.md).

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

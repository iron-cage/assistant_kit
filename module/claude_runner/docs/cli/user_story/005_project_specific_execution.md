# Run Claude scoped to a project directory with isolated session storage

**Persona:** Developer working across multiple projects who needs Claude to operate in a specific project directory with isolated session state.
**Goal:** Run Claude scoped to a specific project directory and session storage location so context does not bleed between projects.
**Benefit:** Prevents cross-project context contamination and keeps Claude's working context relevant.
**Priority:** High

### Acceptance Criteria

- `--dir <path>` sets the subprocess working directory; Claude sees the given path as `cwd`
- Session storage is automatically isolated per project directory — derived from `--dir` (or cwd) via `Df()` encoding, with no manual override needed
- `--session-dir <path>` is deprecated and inert (BUG-493); it no longer affects session storage and only emits a deprecation warning
- `--new-session` at the start of a new project task discards the previous session at that location

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`run`](../command/01_run.md) | Default command; `--dir` scopes execution and (via automatic derivation) session storage |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | `--dir` is the runner control flag that drives project isolation |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 6 | [`--interactive`](../param/006_interactive.md) | Continue interactively in the project directory |
| 7 | [`--new-session`](../param/007_new_session.md) | Discard prior session at that location |
| 8 | [`--dir`](../param/008_dir.md) | Set subprocess working directory |
| 10 | [`--session-dir`](../param/010_session_dir.md) | Deprecated, inert (BUG-493) — session storage is now automatic |

### Workflow Steps

1. `clr --dir /path/to/project "task"` — run Claude with the project directory as working directory; session storage is automatically isolated to that project
2. `clr --dir /path/to/project --from /path/to/other-project "task"` — cross-load session history from another project directory into this one
3. `clr --dir /path/to/project --new-session "task"` — start a new task session in that project directory

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 22 | [022_session_isolation_subdir.md](022_session_isolation_subdir.md) | `--subdir` for task-level session isolation within the same project dir |

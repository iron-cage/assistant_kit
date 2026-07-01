# Isolate Claude sessions by named subdirectory

**Persona:** Developer working on multiple parallel tasks in the same project who needs Claude sessions isolated by task name without changing the base project directory or managing session paths manually.
**Goal:** Run Claude in a named subdirectory of the current project directory so each task maintains its own conversation history without any `--session-dir` bookkeeping.
**Benefit:** Keeps per-task conversation histories separate without managing session paths manually.
**Priority:** Medium

### Acceptance Criteria

- AC-001: `--subdir NAME` appends `/-NAME` to the base directory (`--dir` or cwd) to produce the effective execution directory
- AC-002: The effective directory is created automatically before subprocess spawn (no manual `mkdir` needed); in `--dry-run` mode, directory creation is suppressed — dry-run is side-effect-free
- AC-003: Different `--subdir` values under the same `--dir` produce independent Claude session histories
- AC-004: `--subdir .` (explicit default) leaves the base directory unchanged — identity semantics
- AC-005: `CLR_SUBDIR=NAME` env var is equivalent to `--subdir NAME`; CLI flag wins when both are present

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`run`](../command/01_run.md) | Default command; `--subdir` scopes the execution directory |
| 5 | [`ask`](../command/05_ask.md) | `--subdir` applies; same directory-scoping behavior |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | `--subdir` is a runner control flag |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 8 | [`--dir`](../param/008_dir.md) | Base directory to which the subdirectory is appended |
| 28 | [`--subdir`](../param/028_subdir.md) | Named subdirectory appended to base dir |

### Workflow Steps

1. `clr --subdir auth "Fix authentication bug"` — scope the session to a named subdirectory
2. `clr ask --subdir refactor "Explain this module"` — subdirectory isolation in ask mode
3. `CLR_SUBDIR=auth clr "task"` — set subdirectory via environment variable
4. `clr --subdir . "task"` — explicit default; base directory unchanged

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 5 | [005_project_specific_execution.md](005_project_specific_execution.md) | `--dir` for base project scoping; `--subdir` adds task-level isolation within that project |

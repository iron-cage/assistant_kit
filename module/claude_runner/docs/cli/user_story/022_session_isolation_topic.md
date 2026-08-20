# Isolate Claude sessions by named topic directory

**Persona:** Developer working on multiple parallel tasks in the same project who needs Claude sessions isolated by task name without changing the base project directory or managing session paths manually.
**Goal:** Run Claude in a named topic directory of the current project directory so each task maintains its own conversation history, isolated automatically without manual session-path bookkeeping — `--session-dir`, the old manual mechanism, is now deprecated and inert (BUG-493).
**Benefit:** Keeps per-task conversation histories separate without managing session paths manually.
**Priority:** Medium

### Acceptance Criteria

- AC-001: `--topic NAME` appends `/-NAME` to the base directory (`--dir` or cwd) to produce the effective execution directory
- AC-002: The effective directory is created automatically before subprocess spawn (no manual `mkdir` needed); in `--dry-run` mode, directory creation is suppressed — dry-run is side-effect-free
- AC-003: Different `--topic` values under the same `--dir` produce independent Claude session histories
- AC-004: `--topic .` (explicit default) leaves the base directory unchanged — identity semantics
- AC-005: `CLR_TOPIC=NAME` env var is equivalent to `--topic NAME`; CLI flag wins when both are present

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`run`](../command/01_run.md) | Default command; `--topic` scopes the execution directory |
| 5 | [`ask`](../command/05_ask.md) | `--topic` applies; same directory-scoping behavior |
| 11 | [`topic`](../command/11_topic.md) | `--topic` applies; default diverges to an auto-generated slug instead of `.` |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | `--topic` is a runner control flag |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 8 | [`--dir`](../param/008_dir.md) | Base directory to which the topic directory is appended |
| 28 | [`--topic`](../param/028_topic.md) | Named topic directory appended to base dir |

### Workflow Steps

1. `clr --topic auth "Fix authentication bug"` — scope the session to a named topic directory
2. `clr ask --topic refactor "Explain this module"` — topic directory isolation in ask mode
3. `CLR_TOPIC=auth clr "task"` — set topic directory via environment variable
4. `clr --topic . "task"` — explicit default; base directory unchanged

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 5 | [005_project_specific_execution.md](005_project_specific_execution.md) | `--dir` for base project scoping; `--topic` adds task-level isolation within that project |
| 30 | [030_topic_creation.md](030_topic_creation.md) | `topic` auto-generates `--topic`'s value instead of requiring it named explicitly |

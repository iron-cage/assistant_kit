# CLI User Story: Fresh Session

### Scope

- **Purpose**: Document starting a new conversation without prior session context using --new-session.
- **Responsibility**: Define acceptance criteria for omitting default session continuation.
- **In Scope**: --new-session suppressing -c, clean slate start, remaining defaults unchanged, both modes.
- **Out of Scope**: Session continuation (default behavior, covered in 001_interactive_repl.md).

### Persona

Developer switching to an entirely unrelated task who wants no prior conversation context to influence Claude's responses.

### Goal

Start a genuinely new Claude conversation without session continuation so prior context does not bleed into the new task.

### Acceptance Criteria

- `--new-session` omits the default `-c` from the subprocess call
- No prior session context is loaded; Claude starts with a clean slate
- All other defaults remain in effect: `--dangerously-skip-permissions`, `--effort max`, `ultrathink` suffix
- Works in both print mode and interactive mode

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`run`](../command/01_run.md) | Default command; `--new-session` suppresses continuation |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | `--new-session` is a runner control flag |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 3 | [`--model`](../param/003_model.md) | Optional: choose model for the fresh task |
| 7 | [`--new-session`](../param/007_new_session.md) | Suppress default session continuation |
| 8 | [`--dir`](../param/008_dir.md) | Optional: set project directory for the new task |

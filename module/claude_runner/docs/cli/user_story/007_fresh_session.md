# User Story :: 007. Fresh Session

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

| # | Command | Notes |
|---|---------|-------|
| 1 | [`run`](../command.md#command--1-run) | `--new-session` suppresses default `-c` |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`--new-session`](../param/07_new_session.md) | Suppress default session continuation |
| 2 | [`--model`](../param/03_model.md) | Optional: choose model for the fresh task |
| 3 | [`--dir`](../param/08_dir.md) | Optional: set project directory for the new task |

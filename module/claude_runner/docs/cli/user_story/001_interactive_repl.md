# User Story :: 001. Interactive REPL

### Persona

Developer exploring a codebase or continuing an ongoing task; wants to converse with Claude interactively over multiple turns without specifying an up-front prompt.

### Goal

Open the Claude Code REPL with automatic session continuation so the conversation context from the last session is immediately available.

### Acceptance Criteria

- `clr` with no message launches the interactive REPL (stdin/stdout connected to subprocess)
- Session continues from the most recent conversation automatically (`-c` injected by default)
- `--dir <path>` sets the subprocess working directory for project-specific REPL sessions
- `--new-session` discards prior context and opens a fresh conversation

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [`run`](../command.md#command--1-run) | Default command; no message triggers REPL mode |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`--dir`](../param/08_dir.md) | Set working directory for the REPL session |
| 2 | [`--new-session`](../param/07_new_session.md) | Discard prior context; start fresh |
| 3 | [`--session-dir`](../param/10_session_dir.md) | Override session storage location |

# CLI User Story: Interactive REPL

### Scope

- **Purpose**: Document the interactive REPL invocation with automatic session continuation.
- **Responsibility**: Define acceptance criteria for opening the REPL without an explicit message.
- **In Scope**: No-message REPL launch, session continuation, --dir project scoping, --new-session override.
- **Out of Scope**: Message-driven invocation (→ 003_interactive_with_message.md), print mode (→ 002_print_mode_capture.md).

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

| # | Command | Role |
|---|---------|------|
| 1 | [`run`](../command/01_run.md) | Default command; no message triggers REPL mode |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | `--dir`, `--new-session`, `--session-dir` control REPL session |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 7 | [`--new-session`](../param/007_new_session.md) | Discard prior context; start fresh |
| 8 | [`--dir`](../param/008_dir.md) | Set working directory for the REPL session |
| 10 | [`--session-dir`](../param/010_session_dir.md) | Override session storage location |

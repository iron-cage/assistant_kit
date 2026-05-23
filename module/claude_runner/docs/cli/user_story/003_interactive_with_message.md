# User Story :: 003. Interactive With Message

### Persona

Developer who wants to send an initial prompt but remain in an interactive TTY session to continue the conversation manually rather than receive a single captured response.

### Goal

Send an initial message to Claude and stay in interactive mode for follow-up turns.

### Acceptance Criteria

- `--interactive` with a message opts out of the default print mode
- TTY stdin/stdout is connected directly to the subprocess (passthrough)
- Initial message is sent before the interactive session opens
- Session continues from the previous conversation by default; `--new-session` for fresh start

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [`run`](../001_command.md#command--1-run) | `--interactive` activates TTY passthrough |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`--interactive`](../param/006_interactive.md) | Opt out of print mode; enable TTY passthrough |
| 2 | [`[MESSAGE]`](../param/001_message.md) | Initial prompt sent before the REPL session |
| 3 | [`--dir`](../param/008_dir.md) | Set working directory for the session |
| 4 | [`--new-session`](../param/007_new_session.md) | Discard prior context |

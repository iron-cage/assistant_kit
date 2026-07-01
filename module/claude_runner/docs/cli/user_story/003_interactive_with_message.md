# Send an initial message and stay in interactive mode

**Persona:** Developer who wants to send an initial prompt but remain in an interactive TTY session to continue the conversation manually rather than receive a single captured response.
**Goal:** Send an initial message to Claude and stay in interactive mode for follow-up turns.
**Benefit:** Combines the immediacy of a pre-loaded prompt with the flexibility of a full REPL session.
**Priority:** Medium

### Acceptance Criteria

- `--interactive` with a message opts out of the default print mode
- TTY stdin/stdout is connected directly to the subprocess (passthrough)
- Initial message is sent before the interactive session opens
- Session continues from the previous conversation by default; `--new-session` for fresh start

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`run`](../command/01_run.md) | Default command; `--interactive` overrides print mode |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | `--interactive` is a runner control flag |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`--message`](../param/001_message.md) | Initial prompt sent before the REPL session |
| 6 | [`--interactive`](../param/006_interactive.md) | Opt out of print mode; enable TTY passthrough |
| 7 | [`--new-session`](../param/007_new_session.md) | Discard prior context |
| 8 | [`--dir`](../param/008_dir.md) | Set working directory for the session |

### Workflow Steps

1. `clr --interactive "initial prompt"` — send an opening message and stay in the REPL
2. `clr --interactive --new-session "opening message"` — start fresh with an initial message

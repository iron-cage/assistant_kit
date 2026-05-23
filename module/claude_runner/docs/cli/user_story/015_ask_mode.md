# User Story :: 015. Ask Mode

### Persona

Developer who needs a quick answer to a question — about code, concepts, errors, or
documentation — without starting an autonomous task, modifying files, or persisting
session state.

### Goal

Send a single-turn question to Claude and get a clean, direct answer on stdout, with
no tool use, no session continuation, and no extended thinking unless explicitly
overridden.

### Acceptance Criteria

- `clr ask "What does this function do?"` runs Claude in print mode with no `-c`,
  no `--dangerously-skip-permissions`, no ultrathink suffix, effort `high`, and
  max-tokens 16,384 by default
- Print mode is always on for `ask` regardless of whether a message is given
- No session state is persisted (`--no-persist` default ON for ask)
- No chrome browser context (`--no-chrome` default ON for ask)
- All 25 parameters from `run` are accepted and can override the ask defaults:
  `clr ask --effort max "..."`, `clr ask --max-tokens 200000 "..."`, etc.
- `clr ask --dry-run "What is X?"` shows assembled command reflecting ask defaults

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [`ask`](../001_command.md#command--5-ask) | Lightweight Q&A facade of `run` |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`[MESSAGE]`](../param/001_message.md) | Question text for Claude |
| 2 | [`--effort`](../param/017_effort.md) | Default `high` for ask (overridable) |
| 3 | [`--max-tokens`](../param/009_max_tokens.md) | Default 16,384 for ask (overridable) |
| 4 | [`--no-ultrathink`](../param/014_no_ultrathink.md) | Default ON for ask |
| 5 | [`--no-skip-permissions`](../param/005_no_skip_permissions.md) | Default ON for ask (no bypass) |
| 6 | [`--new-session`](../param/007_new_session.md) | Default ON for ask (no continuation) |
| 7 | [`--file`](../param/025_file.md) | Pipe file content as stdin to ask |

### Related User Stories

| # | User Story | Relationship |
|---|-----------|-------------|
| 1 | [002 Print Mode Capture](002_print_mode_capture.md) | `ask` always uses print mode |
| 2 | [007 Fresh Session](007_fresh_session.md) | `ask` defaults to new session (no `-c`) |

# CLI User Story: Ask Mode

### Scope

- **Purpose**: Document the `ask` command as a lightweight Q&A facade with conservative defaults.
- **Responsibility**: Define acceptance criteria for ask-specific defaults and overridability.
- **In Scope**: Ask defaults (print mode always on, no -c, no permissions bypass, no ultrathink, effort high, max-tokens 16384), all run params accepted.
- **Out of Scope**: General run mode (→ 001_interactive_repl.md, 002_print_mode_capture.md).

### Persona

Developer who needs a quick answer to a question — about code, concepts, errors, or documentation — without starting an autonomous task, modifying files, or persisting session state.

### Goal

Send a single-turn question to Claude and get a clean, direct answer on stdout, with no tool use, no session continuation, and no extended thinking unless explicitly overridden.

### Acceptance Criteria

- `clr ask "What does this function do?"` runs Claude in print mode with no `-c`, no `--dangerously-skip-permissions`, no ultrathink suffix, effort `high`, and max-tokens 16,384 by default
- Print mode is always on for `ask` regardless of whether a message is given
- No session state is persisted (`--no-persist` default ON for ask)
- No chrome browser context (`--no-chrome` default ON for ask)
- All 26 parameters from `run` are accepted and can override the ask defaults: `clr ask --effort max "..."`, `clr ask --max-tokens 200000 "..."`, `clr ask --subdir work "..."`, etc.
- `clr ask --dry-run "What is X?"` shows assembled command reflecting ask defaults

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 5 | [`ask`](../command/05_ask.md) | Lightweight Q&A facade with conservative defaults |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | `--effort` default overridden to `high` |
| 2 | [Runner Control](../param_group/02_runner_control.md) | Multiple defaults overridden for lightweight mode |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`--message`](../param/001_message.md) | Question text for Claude |
| 5 | [`--no-skip-permissions`](../param/005_no_skip_permissions.md) | Default ON for ask (no bypass) |
| 7 | [`--new-session`](../param/007_new_session.md) | Default ON for ask (no continuation) |
| 9 | [`--max-tokens`](../param/009_max_tokens.md) | Default 16,384 for ask (overridable) |
| 14 | [`--no-ultrathink`](../param/014_no_ultrathink.md) | Default ON for ask |
| 17 | [`--effort`](../param/017_effort.md) | Default `high` for ask (overridable) |
| 25 | [`--file`](../param/025_file.md) | Pipe file content as stdin to ask |
| 28 | [`--subdir`](../param/028_subdir.md) | Named workspace isolation within ask |

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 2 | [Print Mode Capture](002_print_mode_capture.md) | `ask` always uses print mode |
| 7 | [Fresh Session](007_fresh_session.md) | `ask` defaults to new session (no `-c`) |
| 17 | [Model Selection](017_model_selection.md) | `ask` accepts `--model` to override the default model |
| 20 | [Suppress Effort Max](020_suppress_effort_max.md) | `ask` defaults to `--effort high`; `--no-effort-max` suppresses the run-mode `max` default |

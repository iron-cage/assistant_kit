# User Story :: 009. Custom System Prompt

### Persona

Developer building a domain-specific automation agent who needs to constrain or replace Claude's default behavioral context for a specific task.

### Goal

Replace or extend the default system prompt to shape Claude's behavior for domain-specific automation (e.g. "respond only in JSON", "act as a Rust expert").

### Acceptance Criteria

- `--system-prompt` replaces the built-in system prompt entirely; tool definitions still survive
- `--append-system-prompt` adds text on top of the default system prompt (lighter touch)
- Both flags can be combined: replace first, then append
- `--append-system-prompt` is the safer default recommendation; `--system-prompt` is the escape hatch

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [`run`](../001_command.md#command--1-run) | Both system prompt flags apply to `run` |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`--system-prompt`](../param/015_system_prompt.md) | Full replacement of the system prompt |
| 2 | [`--append-system-prompt`](../param/016_append_system_prompt.md) | Additive extension of the default |

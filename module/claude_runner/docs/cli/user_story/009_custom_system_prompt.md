# Replace or extend the system prompt for domain-specific automation

**Persona:** Developer building a domain-specific automation agent who needs to constrain or replace Claude's default behavioral context for a specific task.
**Goal:** Replace or extend the default system prompt to shape Claude's behavior for domain-specific automation (e.g. "respond only in JSON", "act as a Rust expert").
**Benefit:** Enables domain-specific automation by shaping Claude's behavior context without modifying the user-turn prompt.
**Priority:** Medium

### Acceptance Criteria

- `--system-prompt` replaces the built-in system prompt entirely; tool definitions still survive
- `--append-system-prompt` adds text on top of the default system prompt (lighter touch)
- Both flags can be combined: replace first, then append
- `--append-system-prompt` is the safer default recommendation; `--system-prompt` is the escape hatch

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`run`](../command/01_run.md) | Default command; system prompt flags modify behavioral context |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 3 | [System Prompt](../param_group/03_system_prompt.md) | Both system prompt flags belong to this group |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 15 | [`--system-prompt`](../param/015_system_prompt.md) | Full replacement of the system prompt |
| 16 | [`--append-system-prompt`](../param/016_append_system_prompt.md) | Additive extension of the default |

### Workflow Steps

1. `clr --system-prompt "You are a Rust expert." "Review this function"` — replace the built-in system prompt
2. `clr --append-system-prompt "Always respond in JSON." "List failing tests"` — extend the default system prompt
3. `clr --system-prompt "Base prompt." --append-system-prompt "Additional constraint." "task"` — replace then append

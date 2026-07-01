# CLI Parameter Group: System Prompt

**Pattern:** Forwarded to the `claude` subprocess via `--system-prompt` / `--append-system-prompt` flags; distinct from Claude-Native (content-injection, not control).

**Purpose:** Inject or extend the behavioral system context sent to the `claude` subprocess.
**Order:** 3

Although forwarded to claude (like Claude-Native Flags), they form a dedicated
group to keep parameter ranges contiguous: params 15–16 cannot join Group 1's
params 2–4 without introducing a gap in the range.

### Semantic Coherence Test

"Is this flag used to inject or extend the system prompt sent to claude?" — YES for both.

### Why NOT X

- `--model`, `--print`, `--verbose`: Claude-native but not system-prompt related
- All Runner Control flags: consumed by the runner, not forwarded to claude
- `[MESSAGE]`: user-turn content, not system-turn context

### Invariants

Both parameters produce system-prompt-related subprocess flags (`--system-prompt` or
`--append-system-prompt`). Neither is runner-only.

### Notes

—

### Typical Patterns

```sh
clr --system-prompt "You are a Rust expert." "Review this PR"
clr --append-system-prompt "Always respond in JSON." "List failing tests"
```

### Referenced Commands

| # | Command | Membership | Excluded Params | Notes |
|---|---------|------------|-----------------|-------|
| 1 | [`run`](../command/01_run.md) | Full | — | Both params apply; default command |
| 5 | [`ask`](../command/05_ask.md) | Full | — | Both params apply; only defaults differ |

### Referenced Parameters

| Parameter | Type | Default | Role in Group | Description |
|-----------|------|---------|---------------|-------------|
| [`--system-prompt`](../param/015_system_prompt.md) | [`SystemPromptText`](../type/06_system_prompt_text.md) | — | Full replacement | Set system prompt (replaces the default) |
| [`--append-system-prompt`](../param/016_append_system_prompt.md) | [`SystemPromptText`](../type/06_system_prompt_text.md) | — | Additive extension | Append text to the default system prompt |

### Referenced Tests

| # | Test Spec | Scope |
|---|-----------|-------|
| 3 | [03_system_prompt.md](../../../tests/docs/cli/param_group/03_system_prompt.md) | System Prompt group behavior |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 9 | [009_custom_system_prompt.md](../user_story/009_custom_system_prompt.md) | Developer |

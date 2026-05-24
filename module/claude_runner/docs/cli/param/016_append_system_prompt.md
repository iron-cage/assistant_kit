# CLI Parameter: --append-system-prompt

Append text to the default system prompt. Additive — does not replace the
built-in system prompt. When omitted, nothing is appended.

- **Type:** [`SystemPromptText`](../type/06_system_prompt_text.md)
- **Default:** — (nothing appended when absent)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [System Prompt](../param_group/03_system_prompt.md)
- **Validation:** requires a value; `--append-system-prompt` at end of argv → error

```sh
clr --append-system-prompt "Always respond in JSON." "List failing tests"
```

**Recommended over `--system-prompt` for most use cases.** All built-in Claude Code
behaviors are preserved — safety rules, CLAUDE.md handling, output style, tool usage
policies. The custom text is appended after the full default prompt.

**Precedence vs CLAUDE.md:** `--append-system-prompt` appends directly into the
*system prompt* (highest-priority position). `CLAUDE.md` is injected as the first
*user message* — a different, lower-priority mechanism. When both are active,
`--append-system-prompt` instructions have stronger persistence.

**Note:** Both `--system-prompt` and `--append-system-prompt` may be
given in the same invocation. Both are forwarded to claude in parse order.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`SystemPromptText`](../type/06_system_prompt_text.md) | Semantic | String | any UTF-8 text |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 3 | [System Prompt](../param_group/03_system_prompt.md) | Full | `--system-prompt` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | — | — |
| 5 | [`ask`](../command/05_ask.md) | — | — |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 9 | [009_custom_system_prompt.md](../user_story/009_custom_system_prompt.md) | Developer |

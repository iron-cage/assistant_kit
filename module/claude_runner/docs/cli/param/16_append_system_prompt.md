# Parameter :: 16. `--append-system-prompt`

Append text to the default system prompt. Additive — does not replace the
built-in system prompt. When omitted, nothing is appended.

- **Type:** [`SystemPromptText`](../type.md#type--6-systemprompttext)
- **Default:** — (nothing appended when absent)
- **Command:** [`run`](../command.md#command--1-run)
- **Group:** [System Prompt](../param_group.md#group--3-system-prompt)
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

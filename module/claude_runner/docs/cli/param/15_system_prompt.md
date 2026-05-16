# Parameter :: 15. `--system-prompt`

Replace the default system prompt sent to the `claude` subprocess with a
custom text. When omitted, Claude's built-in system prompt remains in effect.

- **Type:** [`SystemPromptText`](../type.md#type--6-systemprompttext)
- **Default:** — (built-in system prompt unchanged when absent)
- **Command:** [`run`](../command.md#command--1-run)
- **Group:** [System Prompt](../param_group.md#group--3-system-prompt)
- **Validation:** requires a value; `--system-prompt` at end of argv → error

```sh
clr --system-prompt "You are a Rust expert. Be concise." "Review PR"
clr --dry-run --system-prompt "Be concise." "test"   # preview the flag
```

**What is preserved after replacement:** Tool definitions (~12,000 tokens covering
Bash, Read, Write, Edit, Glob, Grep, WebFetch, etc.) are injected into the assembled
prompt before the replacement is applied and survive intact. Claude can still call
all tools normally.

**What is lost after replacement:** The entire behavioral layer — Claude Code's coding
guidelines, git safety rules, security constraints, output style ("no emojis", conciseness),
CLAUDE.md-handling instructions, environment/project context, and sub-agent coordination
prompts. Claude has raw tool access but no guidance on when or how to use tools safely.

**Use case:** Specialized single-purpose agents that need complete control over behavior
and are prepared to re-specify everything Claude Code normally handles automatically.
For most use cases, `--append-system-prompt` is the correct choice.

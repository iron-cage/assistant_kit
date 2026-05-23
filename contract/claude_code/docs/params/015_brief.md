# brief

Enables the `SendUserMessage` tool, allowing sub-agents to communicate directly with the user.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--brief` |
| Env Var | — |
| Config Key | — |

### Type

bool

### Default

`off`

### Description

Enables the `SendUserMessage` tool, which allows sub-agents to communicate directly with the user during agentic workflows. Normally sub-agents cannot send messages to the user — only the top-level session can. With `--brief`, agents can surface questions or status updates without the top-level orchestrator acting as intermediary.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
# max_turns

Limits the number of agentic turns Claude Code may take before stopping.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--max-turns <n>` |
| Env Var | — |
| Config Key | — |

### Type

integer (positive)

### Default

— (unlimited; Claude Code runs until task completion or budget exhaustion)

### Description

Caps the number of agentic turns Claude Code may execute in a single session. Each tool call + response constitutes one turn. When the limit is reached, Claude Code stops and returns whatever progress was made. Useful for bounding cost and preventing runaway sessions.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |

# prompt

The initial text message sent to Claude as the user's first turn.

### Forms

| | Value |
|-|-------|
| CLI Flag | `<message>` (positional argument) |
| Env Var | — |
| Config Key | — |

### Type

string

### Default

— (required when not in interactive mode)

### Description

The text message sent to Claude as the initial user prompt. Provided as a bare positional argument — no flag name precedes it. When omitted, Claude enters interactive REPL mode. Mutually exclusive with interactive REPL; supplying a message implies print mode by default.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
# append_system_prompt

Appends the supplied text to the end of the default system prompt.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--append-system-prompt <prompt>` |
| Env Var | — |
| Config Key | — |

### Type

string

### Default

—

### Description

Appends the supplied text to the end of the default system prompt. The default prompt remains intact; the appended text is added after it. Useful for injecting per-invocation context (e.g. project conventions, user identity) without discarding the baseline instructions. Cannot be combined with `--system-prompt`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
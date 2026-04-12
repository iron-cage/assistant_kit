# system_prompt

Replaces the entire default system prompt with the supplied text.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--system-prompt <prompt>` |
| Env Var | — |
| Config Key | — |

### Type

string

### Default

—

### Description

Replaces the entire default system prompt with the supplied text. The default system prompt provides Claude's persona, tool descriptions, and workspace context — replacing it entirely removes all of that unless the replacement text re-establishes it. For adding to the default rather than replacing it, use `--append-system-prompt`. Cannot be combined with `--append-system-prompt` on the same invocation.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
# verbose

Enables verbose output mode, emitting additional internal reasoning and tool call details.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--verbose` |
| Env Var | — |
| Config Key | — |

### Type

bool

### Default

`off`

### Description

Enables verbose output mode, overriding whatever the config file specifies. When on, Claude emits additional internal reasoning steps and tool call details to stderr. Useful for debugging sessions or understanding what Claude is doing. The flag takes no value — its presence enables verbose mode.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
# Parameter: bash_max_timeout_ms

### Forms

| Form | Value |
|------|-------|
| Env Var | `BASH_MAX_TIMEOUT_MS` |

### Type

integer (milliseconds)

### Default

`600000` (10 minutes)

### Description

Maximum timeout the model can set for Bash tool commands. Even if the model
requests a longer timeout, it will be capped at this value. Controls the upper
bound of the Bash tool's `timeout` parameter.

### Since

v0.2.108 (2025-05-13)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [096_bash_default_timeout_ms.md](096_bash_default_timeout_ms.md) | Default timeout |
| doc | [../tool/004_bash.md](../tool/004_bash.md) | Bash tool |

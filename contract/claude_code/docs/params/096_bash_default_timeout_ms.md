# Parameter: bash_default_timeout_ms

### Forms

| Form | Value |
|------|-------|
| Env Var | `BASH_DEFAULT_TIMEOUT_MS` |

### Type

integer (milliseconds)

### Default

`120000` (2 minutes)

### Description

Default timeout for long-running Bash tool commands. Applied when the model does
not specify an explicit timeout. The model can request a longer timeout up to
`BASH_MAX_TIMEOUT_MS`.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [098_bash_max_timeout_ms.md](098_bash_max_timeout_ms.md) | `BASH_MAX_TIMEOUT_MS` — maximum timeout cap |
| doc | [013_bash_timeout.md](013_bash_timeout.md) | `CLAUDE_CODE_BASH_TIMEOUT` — runner-level equivalent |
| doc | [../tool/004_bash.md](../tool/004_bash.md) | Bash tool |

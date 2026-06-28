# Parameter: bash_max_output_length

### Forms

| Form | Value |
|------|-------|
| Env Var | `BASH_MAX_OUTPUT_LENGTH` |

### Type

integer (characters)

### Default

Not documented

### Description

Maximum number of characters in Bash tool output before the result is saved to a
file instead of being returned inline. When output exceeds this limit, it is
written to a temporary file and a path reference is returned to the model.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [../tool/004_bash.md](../tool/004_bash.md) | Bash tool |

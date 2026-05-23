# max_output_tokens

Sets the maximum number of tokens Claude may generate in a single response.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_MAX_OUTPUT_TOKENS` |
| Config Key | — |

### Type

integer

### Default

`32 000`

### Description

Sets the maximum number of tokens Claude may generate in a single response. Responses are truncated at this limit. The binary default is 32 000; the `claude_runner_core` builder raises this to 200 000 for automation use cases where longer outputs are expected. Increase for tasks that produce large files or long analyses.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
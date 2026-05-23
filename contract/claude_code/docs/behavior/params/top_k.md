# top_k

Top-k sampling cutoff limiting token candidates to the k highest-probability options.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_TOP_K` |
| Config Key | — |

### Type

integer

### Default

— (model default when unset)

### Description

Top-k sampling cutoff. Limits token sampling to the k highest-probability tokens at each step. When unset, the model's default top-k is used. Lower values make output more predictable; higher values allow more diversity. Generally not tuned independently — prefer `temperature` or `top_p` for most use cases.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
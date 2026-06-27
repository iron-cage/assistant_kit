# top_p

Nucleus sampling threshold controlling which tokens are candidates at each step.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_TOP_P` |
| Config Key | — |

### Type

float — valid range: 0.0–1.0

### Default

— (model default when unset)

### Since

pre-v1.0 (unverified)

### Description

Nucleus sampling threshold. At each token, only the top-probability tokens whose cumulative probability reaches `top_p` are considered. Lower values restrict sampling to a smaller, higher-confidence token set. When unset, the model's default top-p is used. Generally, tune either temperature or top_p but not both simultaneously.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [065_temperature.md](065_temperature.md) | Sampling temperature (sibling sampling parameter) |
| doc | [069_top_k.md](069_top_k.md) | Top-k sampling (sibling sampling parameter) |
# fallback_model

Specifies an alternative model to use automatically when the primary model is unavailable.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--fallback-model <model>` |
| Env Var | — |
| Config Key | — |

### Type

string

### Default

—

### Description

Specifies an alternative model to use automatically when the primary model (`--model`) is overloaded or unavailable. Accepts the same alias and full-ID formats as `--model`. Only active in print mode (`--print`). When the primary model is available, the fallback is never used. Useful for high-availability automation pipelines.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
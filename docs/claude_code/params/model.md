# model

Specifies the Claude model to use for the session.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--model <model>` |
| Env Var | — |
| Config Key | — |

### Type

string

### Default

`claude-sonnet-4-6`

### Description

Specifies the model to use for this session. Accepts short aliases (`sonnet`, `opus`, `haiku`) or full model IDs (e.g. `claude-sonnet-4-6`). The default resolves to the latest Sonnet model. Overrides any model setting in config files for the duration of the session.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
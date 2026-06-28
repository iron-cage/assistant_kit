# default_sonnet_model

Overrides the default model used when the `sonnet` alias is specified.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `ANTHROPIC_DEFAULT_SONNET_MODEL` |
| Config Key | — |

### Type

string (full model ID)

### Default

Latest Sonnet model (e.g. `claude-sonnet-4-6`)

### Since

v2.1.174

### Description

Overrides which concrete model ID the `sonnet` alias resolves to. By default,
the `sonnet` alias maps to the latest Sonnet model. This env var allows
organizations or users to pin the alias to a specific model version.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [042_model.md](042_model.md) | Model selection |

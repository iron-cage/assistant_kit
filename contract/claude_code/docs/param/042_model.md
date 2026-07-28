# model

Specifies the Claude model to use for the session.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--model <model>` |
| Env Var | — |
| Config Key | `model` |

### Type

string

### Default

`claude-sonnet-5`

### Since

pre-v1.0 (unverified)

### Description

Specifies the model to use for this session. Accepts short aliases (`sonnet`, `opus`, `haiku`) or full model IDs (e.g. `claude-sonnet-5`). The default resolves to the latest Sonnet model. When set as `model` in `~/.claude/settings.json`, persists the model preference across all sessions. CLI flag overrides the config key for the current session.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [079_subagent_model.md](079_subagent_model.md) | Subagent model override |
| doc | [026_fallback_model.md](026_fallback_model.md) | Fallback when primary model is unavailable |
| doc | [085_default_sonnet_model.md](085_default_sonnet_model.md) | Override for the `sonnet` alias resolution |
| doc | [023_effort.md](023_effort.md) | Reasoning effort level (affects model compute) |
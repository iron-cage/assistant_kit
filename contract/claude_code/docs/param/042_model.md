# model

Specifies the Claude model to use for the session.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--model <model>` |
| Env Var | `ANTHROPIC_MODEL` |
| Config Key | `model` |

### Type

string

### Default

`claude-sonnet-5`

### Since

pre-v1.0 (unverified)

### Description

Specifies the model to use for this session. Accepts short aliases (`sonnet`, `opus`, `haiku`) or full model IDs (e.g. `claude-sonnet-5`). The default resolves to the latest Sonnet model. When set as `model` in `~/.claude/settings.json`, persists the model preference across all sessions. `ANTHROPIC_MODEL` overrides the config key, and the CLI flag overrides both for the current session (verified on v2.1.283 by capturing the request's `model` field). `CLAUDE_MODEL` is not read: it never occurs in the v2.1.283 binary, while `ANTHROPIC_MODEL` occurs 24 times.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [079_subagent_model.md](079_subagent_model.md) | Subagent model override |
| doc | [026_fallback_model.md](026_fallback_model.md) | Fallback when primary model is unavailable |
| doc | [085_default_sonnet_model.md](085_default_sonnet_model.md) | Override for the `sonnet` alias resolution |
| doc | [023_effort.md](023_effort.md) | Reasoning effort level (affects model compute) |
| doc | [../version/088_v2_1_187.md](../version/088_v2_1_187.md) | Org model restrictions cover `--model`, `/model` and `ANTHROPIC_MODEL` |
| doc | [../version/075_v2_1_169.md](../version/075_v2_1_169.md) | Settings `env` values such as `ANTHROPIC_MODEL` reach background agents |
# effort

Controls the reasoning effort and compute budget for the session.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--effort <level>` |
| Env Var | — |
| Config Key | `effortLevel` |

### Type

enum — `low` `medium` `high` `max`

### Default

`medium`

### Since

pre-v1.0 (unverified)

### Description

Controls the reasoning effort and compute budget for the session. `low` is fast with minimal deliberation. `medium` is the default balanced mode. `high` applies extended thinking. `max` uses maximum available compute for the hardest problems. Higher effort levels increase latency and API cost. Affects models that support extended thinking. Note: the config key name (`effortLevel`) differs from the CLI flag name (`effort`). When set in `~/.claude/settings.json`, persists the effort preference across all sessions.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [042_model.md](042_model.md) | Model selection (effort applies to selected model) |
| doc | [038_max_output_tokens.md](038_max_output_tokens.md) | Output token limit (related compute budget) |
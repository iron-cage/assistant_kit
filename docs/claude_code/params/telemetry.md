# telemetry

Controls whether Claude Code sends anonymous usage telemetry to Anthropic.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_TELEMETRY` |
| Config Key | — |

### Type

bool

### Default

`true`

### Description

Controls whether Claude Code sends anonymous usage telemetry to Anthropic. Telemetry data is non-identifying usage statistics — it does not include prompt content or file data. The binary default is `true` (opt-out model). Set to `false` to disable. The `claude_runner_core` builder defaults to `false` to respect privacy in automation contexts.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
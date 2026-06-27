# experimental_agent_teams

Enables experimental multi-agent team coordination features.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS` |
| Config Key | — |

### Type

bool

### Default

false

### Since

v2.1.178

### Description

Enables experimental agent teams functionality where multiple agents can
coordinate on tasks. This is an experimental feature flag that may be removed
or changed in future versions.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [079_subagent_model.md](079_subagent_model.md) | Subagent model override |
| doc | [../tool/036_send_message.md](../tool/036_send_message.md) | SendMessage tool (requires this flag) |

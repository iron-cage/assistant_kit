# agents

Defines custom agents as an inline JSON object for the session.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--agents <json>` |
| Env Var | — |
| Config Key | — |

### Type

json object

### Default

—

### Since

pre-v1.0 (unverified)

### Description

Defines custom agents as an inline JSON object for the session. Each key is an agent name; each value is an object with `description` and `prompt` fields. Example: `{"reviewer":{"description":"Reviews code","prompt":"You are a code reviewer"}}`. Defined agents can be invoked within the session or via `--agent`. Supplements any agents defined in settings files.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [003_agent.md](003_agent.md) | Single agent override parameter |
| doc | [../tool/007_agent.md](../tool/007_agent.md) | Agent tool this defines agents for |
| doc | [../subcommand/001_agents.md](../subcommand/001_agents.md) | Agents subcommand — lists configured agents |
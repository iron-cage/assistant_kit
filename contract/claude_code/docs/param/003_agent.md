# agent

Overrides the agent configuration for the current session.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--agent <agent>` |
| Env Var | — |
| Config Key | — |

### Type

string (agent name)

### Default

—

### Since

pre-v1.0 (unverified)

### Description

Overrides the agent configuration for the current session. Accepts a named agent defined in `--agents` or in the agents settings. Different agents have different system prompts, tool sets, and capabilities. When unset, the default agent (general-purpose) is used. Enables switching between specialised agents (e.g. a code reviewer vs a planner) per invocation.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [004_agents.md](004_agents.md) | Custom agent definitions |
| doc | [../tool/007_agent.md](../tool/007_agent.md) | Agent tool this configures |
| doc | [../subcommand/001_agents.md](../subcommand/001_agents.md) | Agents subcommand — lists configured agents |
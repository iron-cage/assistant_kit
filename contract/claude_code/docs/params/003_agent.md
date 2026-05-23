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

### Description

Overrides the agent configuration for the current session. Accepts a named agent defined in `--agents` or in the agents settings. Different agents have different system prompts, tool sets, and capabilities. When unset, the default agent (general-purpose) is used. Enables switching between specialised agents (e.g. a code reviewer vs a planner) per invocation.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
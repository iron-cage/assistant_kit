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

### Description

Defines custom agents as an inline JSON object for the session. Each key is an agent name; each value is an object with `description` and `prompt` fields. Example: `{"reviewer":{"description":"Reviews code","prompt":"You are a code reviewer"}}`. Defined agents can be invoked within the session or via `--agent`. Supplements any agents defined in settings files.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
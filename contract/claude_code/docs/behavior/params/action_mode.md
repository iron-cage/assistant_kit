# action_mode

Controls the default action mode for tool execution across the session.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_ACTION_MODE` |
| Config Key | — |

### Type

enum — `Ask` `Auto` `Plan`

### Default

`Ask`

### Description

Controls the default action mode for tool execution. `Ask` (default) prompts the user before each tool call. `Auto` executes tools without prompting. `Plan` puts Claude in read-only planning mode where it describes what it would do without executing. Can be overridden per-session by `--permission-mode`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
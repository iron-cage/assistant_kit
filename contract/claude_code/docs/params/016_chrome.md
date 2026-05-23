# chrome

Controls the Claude-in-Chrome browser integration.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--chrome` / `--no-chrome` |
| Env Var | — |
| Config Key | — |

### Type

bool

### Default

`off`

### Description

Controls the Claude-in-Chrome browser integration. `--chrome` enables it; `--no-chrome` explicitly disables it. The binary default is off; Claude does not attempt to connect to Chrome unless opted in. When enabled, Claude can interact with the active Chrome tab for web-context-aware assistance. The `claude_runner_core` builder defaults this to on for automation.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
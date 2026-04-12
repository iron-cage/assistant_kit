# sandbox_mode

Enables sandbox mode, restricting certain system-level operations Claude Code can perform.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_SANDBOX_MODE` |
| Config Key | — |

### Type

bool

### Default

`true`

### Description

Enables sandbox mode, which restricts certain system-level operations Claude Code can perform. When true (the default), the process runs with additional isolation constraints. Set to `false` in environments that require unrestricted system access. The `claude_runner_core` builder also defaults this to `true` — no difference between builder and binary default for this parameter.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
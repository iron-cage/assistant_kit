# allowed_tools

Restricts available tools to an explicit allowlist for the session.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--allowed-tools <tools...>` |
| Env Var | — |
| Config Key | `allowedTools` |

### Type

string[] (space or comma separated)

### Default

all tools enabled

### Description

Restricts available tools to an explicit allowlist. Tool names follow the format `ToolName` or `ToolName(pattern:*)` for pattern-restricted variants (e.g. `Bash(git:*)` allows only git-prefixed bash commands). Tools not listed are unavailable for the session. Takes precedence over `--tools` when both are provided.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
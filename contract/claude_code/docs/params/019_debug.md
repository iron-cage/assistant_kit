# debug

Enables debug mode with optional category filtering.

### Forms

| | Value |
|-|-------|
| CLI Flag | `-d` / `--debug [filter]` |
| Env Var | — |
| Config Key | — |

### Type

string? (optional category filter)

### Default

`off`

### Description

Enables debug mode. With no argument, all debug categories are emitted. With an optional category filter string (e.g. `"api,hooks"` or `"!1p,!file"`), only matching categories are shown — prefix with `!` to exclude a category. Debug output goes to stderr. Supersedes the deprecated `--mcp-debug` flag for MCP-related debugging.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
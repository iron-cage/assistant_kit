# mcp_debug

> ⚠️ **Deprecated** — use `--debug` (or `--debug=mcp`) instead.

Enables MCP debug mode, showing errors and diagnostic output from MCP servers.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--mcp-debug` |
| Env Var | — |
| Config Key | — |

### Type

bool

### Default

`off`

### Description

Enables MCP (Model Context Protocol) debug mode, showing errors and diagnostic output from MCP servers. This flag is deprecated; use `--debug` with an appropriate category filter to capture MCP-related output. Maintained for backwards compatibility but may be removed in a future release.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
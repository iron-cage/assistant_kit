# mcp_config

Loads MCP server definitions from JSON files or inline JSON strings.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--mcp-config <configs...>` |
| Env Var | — |
| Config Key | — |

### Type

json[] (space-separated file paths or JSON strings)

### Default

—

### Description

Loads MCP (Model Context Protocol) server definitions from JSON files or inline JSON strings. Each config defines one or more MCP servers that Claude can use as tool providers. Multiple configs can be specified space-separated. Combined with `--strict-mcp-config` to disable all other MCP sources and use only these definitions.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
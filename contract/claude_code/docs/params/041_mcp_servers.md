# mcp_servers

Defines MCP server configurations inline in settings without a separate JSON file.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | — |
| Config Key | `mcpServers` |

### Type

object

### Default

`{}`

### Description

Embeds MCP server definitions directly in `settings.json` as an alternative to the `--mcp-config` CLI flag (which points to a separate JSON file). The object keys are server names; each value is an MCP server config object with `command`, `args`, and optional `env` fields. Servers defined here are loaded on every session without needing a `--mcp-config` flag. Typically placed in a project-level `.claude/settings.json` to scope servers to the project, but valid in the global `~/.claude/settings.json` as well.

Example:

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
    }
  }
}
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [mcp_config.md](mcp_config.md) | `--mcp-config` CLI flag (external JSON file form) |

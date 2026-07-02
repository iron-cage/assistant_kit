# strict_mcp_config

Ignores all ambient MCP configs, using only servers defined via `--mcp-config`.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--strict-mcp-config` |
| Env Var | — |
| Config Key | — |

### Type

bool

### Default

`off`

### Since

pre-v1.0 (unverified)

### Description

When set, ignores all MCP server configurations from settings files, project config, and other sources — only the servers defined via `--mcp-config` are available. Useful for hermetic automation where the MCP environment must be fully controlled and predictable, with no ambient MCP servers leaking in from user settings.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [039_mcp_config.md](039_mcp_config.md) | MCP config file (used with this flag) |
| doc | [041_mcp_servers.md](041_mcp_servers.md) | Inline MCP servers (also excluded by this flag) |
| doc | [../subcommand/006_mcp.md](../subcommand/006_mcp.md) | MCP server management subcommand |
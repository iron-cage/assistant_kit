# mcp_config

Load MCP (Model Context Protocol) servers from JSON configuration files or inline JSON strings.

## Type

**CLI** — one or more values (file paths or JSON strings)

## Syntax

```
claude --mcp-config <file-or-json> [<file-or-json> ...]
```

## Default

None (only globally configured MCP servers)

## Description

Model Context Protocol (MCP) extends Claude Code with additional tools from external servers. `--mcp-config` loads additional MCP server configurations for the session without modifying the global config.

Accepts:
- A file path to a JSON config file
- An inline JSON string defining the server configuration

Multiple MCP configs can be specified space-separated. Each adds its servers to the available tool set.

Use `--strict-mcp-config` to ignore all globally configured MCP servers and only use what's specified via this flag.

## MCP Config Format

```json
{
  "mcpServers": {
    "server-name": {
      "command": "npx",
      "args": ["-y", "@org/mcp-server"],
      "env": {
        "API_KEY": "..."
      }
    }
  }
}
```

## Builder API

Use `with_mcp_config()` — Repeated-flag: each call adds one MCP config path or JSON.

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_mcp_config( "/home/user/.config/mcp/servers.json" )
  .with_message( "Use MCP server config" );
```

## Examples

```bash
# Load from file
claude --mcp-config ~/.config/mcp-servers.json --print "Use the database tool"

# Inline JSON config
claude --mcp-config '{"mcpServers":{"fetch":{"command":"npx","args":["-y","@modelcontextprotocol/server-fetch"]}}}' \
  --print "Fetch https://example.com"

# Multiple configs
claude --mcp-config base-mcp.json project-mcp.json --print "Use combined tools"

# Strict: only use the specified config
claude --mcp-config project-mcp.json --strict-mcp-config --print "Isolated environment"
```

## Notes

- MCP servers run as subprocesses; ensure they're installed and accessible
- `--mcp-debug` (deprecated) / `--debug` shows MCP server startup errors
- Session-only: doesn't modify `~/.claude/mcp_servers.json` global config

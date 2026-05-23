# strict_mcp_config

Use only MCP servers from `--mcp-config`, ignoring all other configured MCP sources.

## Type

**CLI** — boolean flag

## Syntax

```
claude --mcp-config <config> --strict-mcp-config
```

## Default

off (all configured MCP servers are available)

## Description

By default, Claude Code loads MCP servers from multiple sources: global config (`~/.claude/`), project config (`.claude/`), and any `--mcp-config` arguments. With `--strict-mcp-config`, only the servers from `--mcp-config` flags are loaded — all other MCP sources are ignored.

Use cases:
- Isolated environments where global MCP servers shouldn't be available
- Reproducible pipelines where the exact tool set must be controlled
- Security-sensitive contexts where only known/approved MCP servers should run
- Testing specific MCP configurations in isolation

## Builder API

Use `with_strict_mcp_config()` — Boolean flag: when `true`, ignores all MCP servers not from `--mcp-config`.

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_mcp_config( "/tmp/servers.json" )
  .with_strict_mcp_config( true )
  .with_message( "Only use explicitly configured MCP servers" );
```

## Examples

```bash
# Only use project-specific MCP, ignore global
claude --mcp-config project-mcp.json --strict-mcp-config \
  --print "Use only the project tools"

# Controlled test environment
claude --mcp-config test-mcp.json --strict-mcp-config --print "Test with mock MCP"
```

## Notes

- Must be combined with at least one `--mcp-config` to be useful; alone it would result in no MCP tools
- Built-in tools (Bash, Read, Write, etc.) are unaffected — this only controls MCP servers
- Useful for CI/CD environments where global developer MCP configs shouldn't leak in

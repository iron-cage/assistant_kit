# mcp_debug

**DEPRECATED** — Enable MCP debug mode showing MCP server errors.

## Type

**CLI** — boolean flag (deprecated)

## Syntax

```
claude --mcp-debug
```

## Default

off

## Deprecation Notice

This flag is deprecated. Use `--debug` or `--debug mcp` instead:

```bash
# Old (deprecated)
claude --mcp-debug

# New (preferred)
claude --debug
claude -d mcp
```

## Description

Formerly enabled debug output specifically for MCP (Model Context Protocol) server interactions, showing startup errors, connection issues, and communication failures.

This has been superseded by the more general `--debug [filter]` flag which covers all debug categories including MCP.

## Migration

Replace all usages of `--mcp-debug`:

| Old | New |
|-----|-----|
| `claude --mcp-debug` | `claude --debug` or `claude -d mcp` |
| `claude --mcp-debug --print "..."` | `claude --debug mcp --print "..."` |

## Notes

- Still accepted by the claude binary but generates a deprecation warning in some versions
- The `--debug` flag provides a superset of what `--mcp-debug` offered
- See `debug.md` for full documentation on the replacement flag

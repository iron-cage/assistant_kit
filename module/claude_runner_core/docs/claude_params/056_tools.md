# tools

Complete override of the available tool set for the session.

## Type

**CLI** — space-separated list

## Syntax

```
claude --tools <tools...>
claude --tools ""
claude --tools "default"
```

## Default

`default` (all built-in tools)

## Description

Fully replaces the available tool set with the specified list. Unlike `--allowed-tools` (which filters the existing set), `--tools` defines the complete set from scratch.

Special values:
- `""` (empty string): Disable all tools — Claude can only respond in text
- `"default"`: Use all standard built-in tools (explicit reset to defaults)
- Tool names: Space-separated list of specific tools

When combined with `--allowed-tools`, both constraints apply.

## Builder API

Use `with_tools()` — Accepts an iterator of tool names to set as the available toolset.

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_tools( [ "Bash", "Read" ] )
  .with_message( "Use only Bash and Read tools" );
```

## Examples

```bash
# No tools — text-only Claude
claude --tools "" --print "What is the time complexity of quicksort?"

# Only bash and read — scripting assistant
claude --tools "Bash Read" --print "Count lines in all .rs files"

# Default set explicitly (useful when config has custom tools)
claude --tools "default" --print "Standard session"
```

## Notes

- `--tools ""` is the most restrictive option: no file access, no shell, no web
- Overrides any tool settings from config files for the session
- MCP tools added via `--mcp-config` are separate and not affected by `--tools`

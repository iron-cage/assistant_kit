# plugin_dir

Load Claude Code plugins from directories for the current session only.

## Type

**CLI** — one or more path values

## Syntax

```
claude --plugin-dir <path> [<path> ...]
```

## Default

None (only globally installed plugins)

## Description

Loads Claude Code plugins from the specified directories for this session without installing them globally. Each directory should contain a valid Claude Code plugin.

Plugins extend Claude Code with new slash commands, tools, and capabilities. This flag allows using plugins for a specific invocation without the need for `claude plugin install`.

Multiple plugin directories can be specified space-separated, and the flag is repeatable.

## Builder API

Use `with_plugin_dir()` — Repeated-flag: each call adds one plugin directory path.

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_plugin_dir( "/home/user/.claude/plugins" )
  .with_message( "Load plugins from custom dir" );
```

## Examples

```bash
# Load a local plugin for this session
claude --plugin-dir ./my-plugin --print "Use the custom tool"

# Multiple plugin directories
claude --plugin-dir ./plugin-a ./plugin-b --print "Use combined plugins"

# Development workflow: test plugin without installing
claude --plugin-dir ./dev/my-plugin --print "Test the new tool"
```

## Notes

- Plugin directories must contain a valid `package.json` with Claude Code plugin configuration
- Session-only: doesn't persist to global plugin installation
- See `claude plugin install` for permanent plugin installation
- Useful for plugin development and testing workflows

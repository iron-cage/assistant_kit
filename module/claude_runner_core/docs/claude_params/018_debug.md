# debug

Enable debug mode with optional category filtering.

## Type

**CLI** — boolean flag with optional string value

## Syntax

```
claude --debug [filter]
claude -d [filter]
```

## Default

off

## Description

Enables debug output, showing internal Claude Code processing information including API calls, tool invocations, hook execution, and system events.

Optionally accepts a filter string to show only specific debug categories:
- Category names: `api`, `hooks`, `file`, `mcp`, `1p` (first-party)
- Comma-separated for multiple: `"api,hooks"`
- Negation prefix `!` to exclude: `"!1p,!file"` — all categories except 1p and file

When `--debug-file <path>` is also set, debug output is written to that file instead of stderr.

## Builder API

Use `with_debug()` — Optional-value: `Some(filter)` adds `--debug filter`, `None` adds `--debug` with no filter.

```rust
use claude_runner_core::ClaudeCommand;

// Debug with category filter
let cmd = ClaudeCommand::new()
  .with_debug( Some( "api" ) )
  .with_message( "Trace API calls" );

// Debug all categories
let cmd = ClaudeCommand::new()
  .with_debug( None::<String> )
  .with_message( "Full debug" );
```

## Examples

```bash
# Enable all debug output
claude --debug "Why is this taking so long?"

# Only API call debug
claude -d api --print "Analyze this"

# Exclude noisy categories
claude --debug "!file,!1p" --print "Debug hooks only"

# Debug to file
claude --debug --debug-file /tmp/claude-debug.log "Trace this session"
```

## Notes

- Debug output goes to stderr by default; use `--debug-file` to redirect to a file
- The `--mcp-debug` flag (deprecated) is superseded by `--debug mcp` or just `--debug`
- Debug mode can generate substantial output for long sessions

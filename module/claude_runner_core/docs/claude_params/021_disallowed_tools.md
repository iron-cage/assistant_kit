# disallowed_tools

Denylist of tools Claude is not permitted to use in the session.

## Type

**CLI** — comma or space-separated list

## Syntax

```
claude --disallowed-tools <tools...>
claude --disallowedTools <tools...>
```

## Default

None (no tools denied)

## Description

Prevents Claude from using the specified tools while keeping all others available. The inverse of `--allowed-tools`.

Tool name syntax is the same as `--allowed-tools`:
- Built-in tools: `Bash`, `Write`, `Edit`, `WebFetch`
- With subcommand restrictions: `Bash(rm:*)` — deny rm commands in Bash
- MCP tools: `mcp__server-name__tool-name`

## Builder API

Use `with_disallowed_tools()` — Accepts an iterator of tool name strings to deny.

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_disallowed_tools( [ "Bash" ] )
  .with_message( "No shell execution allowed" );
```

## Examples

```bash
# Read-only session: block all write operations
claude --disallowed-tools "Write,Edit,Bash" --print "Review this code"

# Block web access
claude --disallowed-tools "WebFetch,WebSearch" --print "Analyze this offline"

# Block specific bash subcommands
claude --disallowed-tools "Bash(rm:*)" --print "Clean up the build"
```

## Notes

- More defensive than `--allowed-tools` — you specify what to block rather than what to allow
- Combined with `--allowed-tools`: both filters apply
- For complete read-only access, blocking `Write`, `Edit`, `Bash`, and `NotebookEdit` is typically sufficient

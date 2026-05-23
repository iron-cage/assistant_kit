# allowed_tools

Allowlist of tools Claude is permitted to use in the session.

## Type

**CLI** — comma or space-separated list

## Syntax

```
claude --allowed-tools <tools...>
claude --allowedTools <tools...>
```

## Default

All built-in tools available

## Description

Restricts Claude to only the specified tools. Tools not in the list are unavailable, even if normally accessible.

Tool names follow the Claude Code tool naming convention:
- Built-in tools: `Bash`, `Read`, `Write`, `Edit`, `Glob`, `Grep`, `WebFetch`, `WebSearch`, `Agent`, `NotebookEdit`
- With subcommand restrictions: `Bash(git:*)` — allow only git commands in Bash
- MCP tools: `mcp__server-name__tool-name`

Accepts comma-separated or space-separated lists.

## Builder API

Use `with_allowed_tools()` — Accepts an iterator of tool name strings (space-separated on CLI).

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_allowed_tools( [ "Bash", "Read", "Write" ] )
  .with_message( "Use only these tools" );
```

## Examples

```bash
# Read-only tools only
claude --allowed-tools "Read,Glob,Grep" --print "Analyze the codebase structure"

# Allow bash but only git commands
claude --allowed-tools "Bash(git:*) Read" --print "What's in the recent commits?"

# Allow specific MCP tool
claude --allowed-tools "mcp__github__create_issue Read" --print "File a bug"
```

## Notes

- Empty string `""` disables all tools (see `--tools ""`)
- Use `--disallowed-tools` to block specific tools while keeping all others available
- `--tools` completely overrides the tool set; `--allowed-tools` is additive filtering

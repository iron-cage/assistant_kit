# auto_approve_tools

Automatically approve all tool execution requests without prompting.

## Type

**Env** — environment variable

## Environment Variable

```
CLAUDE_CODE_AUTO_APPROVE_TOOLS=<bool>
```

Values: `true` / `false`

## Default

`false` (inherits standard — security-sensitive, requires explicit opt-in)

## Description

When enabled, Claude Code approves all tool calls automatically without asking for user confirmation. This is a security-sensitive setting that removes the human-in-the-loop approval step for every tool execution.

The `claude_runner_core` builder keeps the standard `false` default for this parameter — unlike the Tier 1 defaults (max_output_tokens, bash_timeout, etc.) which are overridden for automation convenience, this one stays at `false` because auto-approving all tools is a security decision that must be explicit.

Combine with `dangerously_skip_permissions` for fully autonomous execution. Alternatively, use `permission_mode=acceptEdits` for more granular control (auto-approve edits but not shell commands).

## Builder API

```rust
use claude_runner_core::ClaudeCommand;

// Default: false (inherits standard)
let cmd = ClaudeCommand::new();

// Explicit opt-in for automation
let cmd = ClaudeCommand::new()
  .with_auto_approve_tools( true );
```

Builder method: `with_auto_approve_tools(approve: bool)` — sets `CLAUDE_CODE_AUTO_APPROVE_TOOLS`.

## Examples

```bash
# Enable for trusted automation
CLAUDE_CODE_AUTO_APPROVE_TOOLS=true claude --print "Refactor this module"

# Explicit deny for review session
CLAUDE_CODE_AUTO_APPROVE_TOOLS=false claude "Show me what you would change"
```

## Notes

- **Security-sensitive**: enabling this removes all tool approval prompts
- Unlike `dangerously_skip_permissions` (CLI flag), this is set via environment variable
- For sandboxed CI environments, both `auto_approve_tools=true` and `skip_permissions=true` together provide fully autonomous execution
- See `permission_mode.md` for more granular control

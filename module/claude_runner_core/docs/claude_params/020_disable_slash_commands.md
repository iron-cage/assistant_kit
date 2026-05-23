# disable_slash_commands

Disable all slash command skills for the session.

## Type

**CLI** — boolean flag

## Syntax

```
claude --disable-slash-commands
```

## Default

off (slash commands are enabled)

## Description

Disables all slash command skills (`/commit`, `/dev`, `/review-pr`, etc.) for the session. Claude Code normally provides a rich set of slash commands that trigger predefined skill workflows. This flag disables them entirely, leaving only the raw Claude interaction.

Use cases:
- Environments where slash commands should not be available (e.g., restricted automation contexts)
- When slash commands interfere with custom workflows
- Debugging sessions where you want predictable, un-augmented Claude behavior
- Security contexts where the skill execution capability is undesirable

## Builder API

Use `with_disable_slash_commands()` — Boolean flag.

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_disable_slash_commands( true )
  .with_message( "Run without slash commands" );
```

## Examples

```bash
# Raw Claude session without skills
claude --disable-slash-commands "Help me with this code"

# Scripted session where / shouldn't trigger commands
claude --disable-slash-commands --print "What does /etc/hosts contain?"
```

## Notes

- `/help` and built-in CLI commands (not skills) may still work depending on the claude version
- Skills are the user-invocable commands in the skills directory (e.g., `$PRO/genai/claude/commands/`)
- Does not affect MCP tools or built-in tools (Bash, Read, etc.)

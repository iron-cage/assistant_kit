# brief

Enable the SendUserMessage tool for agent-to-user communication.

## Type

**CLI** — boolean flag

## Syntax

```
claude --brief
```

## Default

off

## Description

Enables the `SendUserMessage` tool, which allows Claude to proactively send messages to the user during agentic workflows. This is distinct from the standard assistant response — it provides a channel for agents to communicate status, ask questions, or provide updates mid-task.

Intended for multi-agent or complex agentic workflows where Claude needs to surface information to the user without completing the current task.

## Builder API

Use `with_brief()` — Boolean flag enabling SendUserMessage for agent sub-sessions.

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_brief( true )
  .with_message( "Run as agent" );
```

## Examples

```bash
# Enable agent communication during long tasks
claude --brief --print "Run the full test suite and report issues as you find them"

# Agentic workflow with user updates
claude --brief "Process all 50 files and check in with me every 10 files"
```

## Notes

- Primarily useful for long-running agentic workflows where mid-task communication matters
- Without `--brief`, Claude can only communicate via the final response
- The `SendUserMessage` tool is a separate channel from the primary task execution

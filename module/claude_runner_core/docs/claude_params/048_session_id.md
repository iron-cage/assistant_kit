# session_id

Use a specific session UUID for the conversation.

## Type

**CLI** — UUID string value

## Syntax

```
claude --session-id <uuid>
```

## Default

Auto-generated UUID for each new session

## Description

Forces Claude Code to use a specific UUID as the session identifier instead of auto-generating one. The UUID must be a valid v4 UUID format (`xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx`).

Use cases:
- Deterministic session IDs for testing/reproducibility
- Pre-assigning a session ID before starting the conversation
- Coordinating session IDs across multiple tools or systems

This is distinct from `--resume` — `--session-id` assigns the ID for a new session, while `--resume` selects an existing session to continue.

## Builder API

Use `with_session_id()` — Accepts a UUID string for the session.

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_session_id( "550e8400-e29b-41d4-a716-446655440000" )
  .with_message( "Run with explicit session ID" );
```

## Examples

```bash
# Start a session with a known UUID
claude --session-id 550e8400-e29b-41d4-a716-446655440000

# Use in automated pipeline for traceability
SESSION_ID=$(uuidgen)
claude --session-id "$SESSION_ID" --print "Process this task"
echo "Session: $SESSION_ID" >> pipeline.log
```

## Notes

- Invalid UUIDs will cause claude to reject the argument
- After the session completes, it can be resumed via `--resume 550e8400-...`
- Useful for audit trails where session IDs must be logged before execution begins

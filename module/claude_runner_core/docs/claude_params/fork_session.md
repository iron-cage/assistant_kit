# fork_session

When resuming, create a new session ID instead of reusing the original.

## Type

**CLI** — boolean flag

## Syntax

```
claude --fork-session --resume <id>
claude --fork-session --continue
```

## Default

off (resuming reuses the original session ID by default)

## Description

When resuming a session (via `--resume` or `--continue`), normally Claude Code appends new conversation turns to the original session file. With `--fork-session`, it creates a new session file (new UUID) initialized with the history from the resumed session.

This is equivalent to branching in version control — the forked session shares history up to the fork point but diverges afterward.

Use cases:
- Trying different approaches from the same starting point
- Creating a read-only "archive" of a conversation while continuing in a fork
- A/B testing different prompts from the same context

## Builder API

Use `with_fork_session()` — Boolean flag: when `true`, creates a new session ID on resume.

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_resume( "abc-123" )
  .with_fork_session( true )
  .with_message( "Fork from this session" );
```

## Examples

```bash
# Fork from most recent session
claude --continue --fork-session

# Fork from a specific session
claude --resume 550e8400-e29b-41d4-a716-446655440000 --fork-session

# Fork and immediately send a different follow-up
claude --resume 550e8400-... --fork-session --print "Try approach B instead"
```

## Notes

- Original session remains intact and resumable
- The new forked session gets its own UUID
- Useful for exploring alternatives without losing the original conversation path

# no_session_persistence

Disable session persistence — sessions will not be saved to disk.

## Type

**CLI** — boolean flag

## Syntax

```
claude --no-session-persistence --print <prompt>
```

## Default

off (sessions are persisted by default)

## Description

Prevents Claude Code from writing session state to `~/.claude/projects/`. The conversation exists only in memory and cannot be resumed after the process exits.

Only works with `--print` mode.

Use cases:
- Privacy-sensitive automation where conversation content must not be stored
- High-throughput pipelines generating many ephemeral queries
- Environments with read-only home directories
- Reducing disk I/O in constrained environments

## Builder API

Use `with_no_session_persistence()` — Boolean flag: when `true`, disables save-to-disk.

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_no_session_persistence( true )
  .with_message( "Ephemeral session — no disk writes" );
```

## Examples

```bash
# Ephemeral query
claude --print --no-session-persistence "What does this regex do: ^[a-z]+$"

# High-throughput pipeline
cat tasks.txt | while read task; do
  claude --print --no-session-persistence "$task"
done
```

## Notes

- Only works with `--print` mode; not applicable to interactive sessions
- No `CLAUDE_CODE_SESSION_DIR` env var equivalent (that sets the location, not disables it)
- Sessions are entirely in-memory; no disk writes occur

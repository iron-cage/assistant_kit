# resume

Resume a conversation by session ID or open interactive picker.

## Type

**CLI** — optional string value

## Syntax

```
claude --resume [session-id-or-search-term]
claude -r [session-id-or-search-term]
```

## Default

None (no session resumed)

## Description

Resumes a previous Claude Code conversation. Accepts:
- A full session UUID: resumes that exact session
- A partial search term: opens interactive picker filtered by that term
- No argument: opens interactive picker showing all recent sessions

Sessions are stored in `~/.claude/projects/`. Each project directory contains session files identified by UUID.

Use `--fork-session` with `--resume` to branch off a new session from the resumed point instead of appending to the original.

Differs from `--continue` which resumes the most recent session automatically without needing an ID.

## Builder API

Use `with_resume()` — Optional-value: `Some(id)` adds `--resume id`, `None` adds `--resume` without ID.

```rust
use claude_runner_core::ClaudeCommand;

// Resume a specific session by ID
let cmd = ClaudeCommand::new()
  .with_resume( Some( "550e8400-e29b-41d4-a716-446655440000" ) )
  .with_message( "Continue this session" );

// Resume the most recent conversation
let cmd = ClaudeCommand::new()
  .with_resume( None::<String> )
  .with_message( "Continue where I left off" );
```

## Examples

```bash
# Resume by exact UUID
claude --resume 550e8400-e29b-41d4-a716-446655440000

# Open picker with search filter
claude --resume "auth-refactor"

# Open picker with no filter
claude -r

# Resume and branch
claude --resume 550e8400-e29b-41d4-a716-446655440000 --fork-session
```

## Notes

- Session UUIDs are visible in `~/.claude/projects/<path>/` directory names
- The interactive picker requires a TTY; not suitable for `--print` mode
- `--resume` and `--continue` should not be combined

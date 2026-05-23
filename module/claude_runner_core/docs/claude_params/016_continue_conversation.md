# continue_conversation

Continue the most recent conversation in the current directory.

## Type

**CLI** — boolean flag

## Syntax

```
claude -c [prompt]
claude --continue [prompt]
```

## Default

off

## Description

Resumes the most recently active Claude Code session in the current working directory. When combined with a prompt, that prompt becomes the next user message in the resumed conversation.

Differs from `--resume <id>` in that it finds the most recent session automatically without requiring a session ID.

Use `--fork-session` alongside `--continue` to branch off a new session ID instead of appending to the original.

## Builder API

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_continue_conversation( true )
  .with_message( "Now refactor the tests too" );
```

Builder method: `with_continue_conversation(continue_: bool)`

The `-c` flag is added before the message argument in the constructed command.

## Examples

```bash
# Continue most recent session interactively
claude --continue

# Continue with a follow-up prompt (non-interactive)
claude --print --continue "Now add error handling"

# Continue and branch to a new session
claude --continue --fork-session
```

## Notes

- "Most recent" is determined by modification time of session state in `~/.claude/projects/`
- If no prior session exists in the directory, behavior depends on claude version (may start fresh or error)
- Combining with `--resume <id>` is not meaningful; use one or the other

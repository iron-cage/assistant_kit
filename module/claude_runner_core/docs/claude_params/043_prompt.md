# prompt

Positional argument: the message/prompt sent to Claude.

## Type

**CLI** — positional argument

## Syntax

```
claude [options] <prompt>
```

## Default

None (starts interactive session if omitted)

## Description

The text prompt passed directly to Claude. When provided, Claude processes it and exits (combined with `--print`) or enters a conversation with that initial message. If omitted entirely, Claude opens an interactive terminal session.

In non-interactive (`--print`) mode, the prompt is the only user turn. In interactive mode, it seeds the first message.

Special handling: quotes are not required by the shell when passing a single word, but multi-word prompts should be quoted to prevent shell word-splitting.

## Builder API

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_message( "Explain this code" );
```

Builder method: `with_message(message: impl Into<String>)`

## Examples

```bash
# Non-interactive: process prompt and exit
claude --print "Summarize this file"

# Interactive: seed first message
claude "Let's refactor the auth module"

# No prompt: start fresh interactive session
claude
```

## Notes

- In `--print` mode, the prompt is required to get a response
- In interactive mode, omitting the prompt shows the welcome screen
- Piped input via `--input-format stream-json` replaces the positional prompt

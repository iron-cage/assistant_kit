# input_format

Input format for `--print` mode.

## Type

**CLI** — enum value

## Syntax

```
claude --print --input-format <format>
```

## Values

| Format | Description |
|--------|-------------|
| `text` | Plain text (default) |
| `stream-json` | Realtime streaming JSON input |

## Default

`text`

## Description

Controls how Claude Code reads input when using `--print` mode. Only works with `--print`.

**`text`**: Standard text input from the positional prompt argument or stdin.

**`stream-json`**: Accepts newline-delimited JSON objects on stdin as input events. This enables real-time streaming input where the user messages are sent as JSON events rather than a single static prompt.

Use `stream-json` for building interactive streaming pipelines where prompts arrive dynamically rather than as a single argument.

Combine with `--output-format stream-json` and `--replay-user-messages` for bidirectional streaming.

## Builder API

Use `with_input_format()` — Accepts an `InputFormat` enum value (`Text` or `StreamJson`).

```rust
use claude_runner_core::{ ClaudeCommand, InputFormat };

let cmd = ClaudeCommand::new()
  .with_input_format( InputFormat::StreamJson )
  .with_message( "Process stream-json input" );
```

## Examples

```bash
# Stream JSON input from a producer
produce_messages | claude --print --input-format stream-json

# Bidirectional stream
produce_messages | \
  claude --print \
    --input-format stream-json \
    --output-format stream-json \
    --replay-user-messages
```

## Notes

- `stream-json` input format requires messages in a specific JSON schema (see `--replay-user-messages` for the format)
- Primarily useful for programmatic pipeline integration
- Combine with `--output-format stream-json` for fully streaming bidirectional communication

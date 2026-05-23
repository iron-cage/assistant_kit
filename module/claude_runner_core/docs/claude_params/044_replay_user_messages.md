# replay_user_messages

Re-emit user messages from stdin back on stdout for acknowledgment.

## Type

**CLI** — boolean flag

## Syntax

```
claude --print --input-format stream-json --output-format stream-json --replay-user-messages
```

## Default

off

## Description

When processing streaming JSON input (`--input-format stream-json`), this flag causes Claude Code to echo each incoming user message event back on stdout as it's received, before emitting the response.

This is used for building streaming pipelines where the consumer needs confirmation that messages were received — essentially an acknowledgment protocol.

Only works with `--input-format=stream-json` and `--output-format=stream-json`.

## Builder API

Use `with_replay_user_messages()` — Boolean flag enabling re-emission of user messages on stdout.

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_replay_user_messages( true )
  .with_message( "Echo user messages too" );
```

## Examples

```bash
# Bidirectional streaming with acknowledgment
echo '{"type":"user","message":{"role":"user","content":"Hello"}}' | \
  claude --print \
    --input-format stream-json \
    --output-format stream-json \
    --replay-user-messages
```

## Notes

- Primarily useful for custom streaming integrations or debugging
- The replay happens immediately when the input event is received, not after processing
- Adds latency overhead for simple use cases; only use when acknowledgment is needed

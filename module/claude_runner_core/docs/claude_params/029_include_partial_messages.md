# include_partial_messages

Include partial message chunks as they arrive in streaming output.

## Type

**CLI** — boolean flag

## Syntax

```
claude --print --output-format stream-json --include-partial-messages
```

## Default

off

## Description

When using `--print` with `--output-format stream-json`, by default only complete messages are emitted as JSON events. With `--include-partial-messages`, partial text chunks are emitted as they stream from the API — enabling true token-by-token streaming output.

This is the flag that enables "typewriter effect" output in custom UIs or streaming consumers.

Only works with `--print` and `--output-format=stream-json`.

## Builder API

Use `with_include_partial_messages()` — Boolean flag enabling partial message chunks in streaming.

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_include_partial_messages( true )
  .with_message( "Stream with partial chunks" );
```

## Examples

```bash
# Stream text tokens as they arrive
claude --print \
  --output-format stream-json \
  --include-partial-messages \
  "Write a detailed explanation of Rust lifetimes" \
  | jq -r 'select(.type == "content_block_delta") | .delta.text // empty'
```

## Notes

- Without this flag, `stream-json` still emits events but waits for complete messages before doing so
- Partial events have `type: "content_block_delta"` in the JSON stream
- Requires `--output-format stream-json` — has no effect with `text` or `json` formats

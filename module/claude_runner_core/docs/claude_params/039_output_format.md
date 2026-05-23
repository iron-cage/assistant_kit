# output_format

Response output format for `--print` mode.

## Type

**CLI** — enum value

## Syntax

```
claude --print --output-format <format>
```

## Values

| Format | Description |
|--------|-------------|
| `text` | Plain text (default) |
| `json` | Single JSON object with full response |
| `stream-json` | Newline-delimited JSON stream (realtime) |

## Default

`text`

## Description

Controls the format of Claude's output when using `--print` mode. Only works with `--print`.

**`text`**: Plain text response, same as what you'd see in the terminal. Suitable for human reading or simple piping.

**`json`**: Single JSON object containing the complete response, metadata, and session information. Waits for the full response before outputting. Useful for programmatic parsing.

**`stream-json`**: Realtime streaming as newline-delimited JSON events. Each event is a JSON object on its own line. Use with `--include-partial-messages` to receive partial text chunks as they arrive.

## Builder API

Use `with_output_format()` — Accepts an `OutputFormat` enum value (`Text`, `Json`, or `StreamJson`).

```rust
use claude_runner_core::{ ClaudeCommand, OutputFormat };

let cmd = ClaudeCommand::new()
  .with_output_format( OutputFormat::Json )
  .with_message( "Return JSON" );
```

## Examples

```bash
# Parse JSON output
claude --print --output-format json "List 3 algorithms" | jq '.result'

# Stream and process events
claude --print --output-format stream-json "Generate a long response" \
  | while IFS= read -r line; do
      echo "$line" | jq -r '.content // empty'
    done

# Text (default)
claude --print "Hello" | grep -i "hello"
```

## Notes

- `stream-json` + `--include-partial-messages` = streaming text chunks as they generate
- JSON format is stable for scripting; text format may change with claude versions
- `stream-json` output format: each line is a complete JSON object with an `event` field

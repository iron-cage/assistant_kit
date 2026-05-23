# betas

Enable beta API features by sending beta headers with API requests.

## Type

**CLI** — one or more string values

## Syntax

```
claude --betas <header> [<header> ...]
```

## Default

None (no beta features enabled)

## Description

Sends the specified beta headers with Anthropic API requests, enabling early-access or experimental features. Multiple beta headers can be specified space-separated.

This flag only works with API key authentication mode (`--api-key` or `ANTHROPIC_API_KEY`). It does not work with OAuth/subscription authentication.

Beta header names correspond to Anthropic's published beta features. Check the Anthropic API documentation for available beta headers.

## Builder API

Use `with_betas()` — Accepts an iterator of beta feature name strings.

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_betas( [ "interleaved-thinking-2025-05-14" ] )
  .with_message( "Use extended thinking" );
```

## Examples

```bash
# Enable a specific beta feature
claude --api-key "$ANTHROPIC_API_KEY" \
  --betas "interleaved-thinking-2025-05-14" \
  --print "Use extended thinking on this problem"

# Multiple beta headers
claude --api-key "$ANTHROPIC_API_KEY" \
  --betas "beta-feature-a" "beta-feature-b" \
  --print "Use both beta features"
```

## Notes

- **API key mode only**: Does not work with subscription/OAuth authentication
- Beta features may change or be removed without notice
- Check Anthropic API changelog and beta documentation for available header values
- Invalid beta headers are typically silently ignored by the API

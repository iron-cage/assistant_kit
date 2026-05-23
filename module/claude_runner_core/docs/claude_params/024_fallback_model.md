# fallback_model

Automatic fallback to a specified model when the primary model is overloaded.

## Type

**CLI** — string value

## Syntax

```
claude --print --fallback-model <model>
```

## Default

None (no fallback; errors on overload)

## Description

When the primary model returns an overload error, Claude Code automatically retries with the specified fallback model. This provides resilience for automation pipelines that can tolerate slightly different model quality in exchange for availability.

Only works with `--print` mode.

Accepts the same model identifiers as `--model`: aliases (`haiku`, `sonnet`, `opus`) or full model IDs.

Typical use: set the primary model to `opus` for quality and fall back to `sonnet` if opus is overloaded.

## Builder API

Use `with_fallback_model()` — Accepts a model alias or full model ID string.

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_fallback_model( "claude-haiku-4-5" )
  .with_message( "Try primary, fall back to haiku" );
```

## Examples

```bash
# Prefer opus, fall back to sonnet
claude --print --model opus --fallback-model sonnet "Analyze this architecture"

# Prefer sonnet, fall back to haiku for high-throughput
claude --print --model sonnet --fallback-model haiku "Summarize this PR"
```

## Notes

- Only triggers on overload errors, not on rate limits or other errors
- The fallback model may produce different output quality/style
- Only works with `--print` mode; interactive sessions do not support automatic fallback

# model

Select the Claude model for the current session.

## Type

**CLI** — string value

## Syntax

```
claude --model <model>
```

## Default

`claude-sonnet-4-6` (current default as of 2026-03)

## Description

Overrides the model used for this session. Accepts both short aliases and full model IDs.

Supported aliases (latest as of knowledge cutoff):
- `opus` → `claude-opus-4-6`
- `sonnet` → `claude-sonnet-4-6`
- `haiku` → `claude-haiku-4-5-20251001`

Full model IDs can also be specified directly (e.g., `claude-sonnet-4-6`).

The model setting from config files is overridden by this flag for the session only.

## Builder API

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_model( "claude-opus-4-6" );

// or with alias
let cmd = ClaudeCommand::new()
  .with_model( "opus" );
```

Builder method: `with_model(model: impl Into<String>)` — adds `--model <value>` to CLI args.

## Examples

```bash
# Use alias
claude --model opus "Write a comprehensive test suite"

# Use full model ID
claude --model claude-haiku-4-5-20251001 --print "Quick question"

# In combination with fallback
claude --print --model sonnet --fallback-model haiku "Analyze this code"
```

## Notes

- Model aliases resolve to specific model IDs at runtime; the alias `sonnet` always maps to the latest Sonnet
- For reproducible automation, prefer full model IDs over aliases
- `--fallback-model` only works with `--print` mode

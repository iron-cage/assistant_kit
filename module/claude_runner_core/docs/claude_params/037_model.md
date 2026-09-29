# model

Select the Claude model for the current session.

## Type

**CLI** — string value

## Syntax

```
claude --model <model>
```

## Default

`claude-sonnet-5` (current default as of 2026-03)

## Description

Overrides the model used for this session. Accepts both short aliases and full model IDs.

Supported aliases (the binary's own alias table, as of v2.1.283):
- `opus` → `claude-opus-5-5` (Foundry: `claude-opus-4-6`; gateway: `claude-opus-4-7`)
- `sonnet` → `claude-sonnet-5` (Bedrock, Vertex, Foundry, Mantle: `claude-sonnet-4-5`; Anthropic-on-AWS, gateway: `claude-sonnet-4-6`)
- `haiku` → `claude-haiku-4-5` (the API alias of `claude-haiku-4-5-20251001`)
- `fable` → `claude-fable-5-1` (gateway: `claude-fable-5`)

The table ships inside the binary and moves between releases: `opus` resolved to `claude-opus-4-8` before v2.1.219 and to `claude-opus-5` in v2.1.219–v2.1.279 ([`013_claude_opus_5.md`](../../../../contract/claude_code/docs/model/013_claude_opus_5.md)). To read it from the installed version:

```bash
V=~/.local/share/claude/versions/$(claude --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')
grep -ao 'aliases:{opus:{default:"[^"]*",per_provider:{[^}]*}' "$V" | head -1
```

Full model IDs can also be specified directly (e.g., `claude-sonnet-5`).

The model setting from config files is overridden by this flag for the session only.

## Builder API

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_model( "claude-opus-4-8" );

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

- Model aliases resolve to specific model IDs at runtime, per provider (see the alias table above) — on Bedrock, `sonnet` is `claude-sonnet-4-5`, not the latest Sonnet
- For reproducible automation, prefer full model IDs over aliases
- `ClaudeCommand::new()` sets no model. `claude_runner_core` exports `DEFAULT_MODEL` (`claude-opus-5-5`): `IsolatedModel::Default` injects it for `run_isolated()`, and `claude_runner` uses it as the `run`/`ask`/`topic` built-in default
- `--fallback-model` only works with `--print` mode

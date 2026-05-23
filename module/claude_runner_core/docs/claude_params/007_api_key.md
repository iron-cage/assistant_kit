# api_key

Anthropic API key for authentication. Settable via CLI flag or environment variable.

## Type

**Both** — CLI flag + environment variable

## Syntax

```bash
# CLI flag
claude --api-key <key>

# Environment variable
ANTHROPIC_API_KEY=sk-ant-... claude ...
```

## Default

None (required for API key auth mode; not required when using OAuth/subscription auth)

## Description

Provides the Anthropic API key used to authenticate requests to the Anthropic API. When set, Claude Code operates in API key mode, billing usage to the associated account.

Two equivalent ways to set it:
1. `--api-key <key>` CLI flag — applies to this invocation only
2. `ANTHROPIC_API_KEY` environment variable — applies to all invocations in the shell session

For production automation, prefer the environment variable so keys aren't visible in process listings.

### Auth modes

Claude Code supports two authentication modes:
- **API key mode**: Provide `ANTHROPIC_API_KEY` or `--api-key`. Billing via API usage.
- **OAuth/subscription mode**: `claude auth login`. No API key needed. Billing via Claude subscription.

## Builder API

```rust
use claude_runner_core::ClaudeCommand;

// Via CLI flag (visible in process list — avoid in production)
let cmd = ClaudeCommand::new()
  .with_api_key( "sk-ant-..." );
```

Builder method: `with_api_key(key: impl Into<String>)` — adds `--api-key <value>` to CLI args.

**Note**: For security, prefer setting `ANTHROPIC_API_KEY` in the environment rather than passing via CLI. The CLI flag value is visible in `ps` output.

## Examples

```bash
# Via environment variable (preferred for automation)
export ANTHROPIC_API_KEY="sk-ant-api03-..."
claude --print "Hello"

# Via CLI flag (less secure, visible in process list)
claude --api-key "sk-ant-..." --print "Hello"

# One-off with env var
ANTHROPIC_API_KEY="sk-ant-..." claude --print "Quick check"
```

## Notes

- `--betas` flag requires API key mode (does not work with OAuth/subscription)
- `ANTHROPIC_API_KEY` is the standard Anthropic SDK env var name
- Key format: `sk-ant-api03-...` (service accounts) or `sk-ant-...` (personal)
- Never commit API keys to version control; use environment variable injection

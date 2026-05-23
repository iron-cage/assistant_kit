# chrome

Toggle Claude-in-Chrome browser integration.

## Type

**CLI** — boolean flags (`--chrome` / `--no-chrome`)

## Syntax

```
claude --chrome
claude --no-chrome
```

## Default

on — `--chrome` is always emitted by the builder. Pass `Some(false)` to suppress, `None` to defer to Claude's own config.

## Description

Enables or disables the Claude-in-Chrome browser integration for the session.

- `--chrome`: Enable the Chrome integration, allowing Claude to interact with the browser
- `--no-chrome`: Disable the Chrome integration, even if enabled in config

The Chrome integration allows Claude Code to read browser context, interact with web pages, and use browser-based tools. This requires the Claude Code Chrome extension to be installed.

## Builder API

Use `with_chrome()` — Tri-state `Option<bool>`: `Some(true)` = `--chrome`, `Some(false)` = `--no-chrome`, `None` = omit flag entirely (defer to Claude's config). The builder default is `Some(true)` — chrome is on unless explicitly overridden.

```rust
use claude_runner_core::ClaudeCommand;

// Default: chrome already on — no call needed
let cmd = ClaudeCommand::new();

// Explicitly disable chrome for this invocation
let cmd = ClaudeCommand::new()
  .with_chrome( Some( false ) );

// Defer to Claude's own config (omit flag entirely)
let cmd = ClaudeCommand::new()
  .with_chrome( None );
```

## Examples

```bash
# Default builder invocation — chrome is on automatically
# (no explicit --chrome needed when using ClaudeCommand::new())

# Disable Chrome for a headless/no-browser invocation
claude --no-chrome --print "Process this without browser access"

# Explicitly enable (equivalent to builder default; useful in raw CLI)
claude --chrome "Debug this JavaScript issue"
```

## Notes

- Requires the Claude Code Chrome extension installed and active
- `--chrome` and `--no-chrome` are mutually exclusive; last one wins
- The Chrome integration is separate from the `WebFetch` and `WebSearch` built-in tools

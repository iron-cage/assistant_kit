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

off (as configured in settings)

## Description

Enables or disables the Claude-in-Chrome browser integration for the session.

- `--chrome`: Enable the Chrome integration, allowing Claude to interact with the browser
- `--no-chrome`: Disable the Chrome integration, even if enabled in config

The Chrome integration allows Claude Code to read browser context, interact with web pages, and use browser-based tools. This requires the Claude Code Chrome extension to be installed.

## Builder API

Use `with_chrome()` — Tri-state `Option<bool>`: `Some(true)` = `--chrome`, `Some(false)` = `--no-chrome`, `None` = omit flag.

```rust
use claude_runner_core::ClaudeCommand;

// Enable chrome integration
let cmd = ClaudeCommand::new()
  .with_chrome( Some( true ) );

// Explicitly disable
let cmd = ClaudeCommand::new()
  .with_chrome( Some( false ) );

// Use config default (no flag emitted)
let cmd = ClaudeCommand::new()
  .with_chrome( None );
```

## Examples

```bash
# Enable Chrome integration for web debugging session
claude --chrome "Debug this JavaScript issue"

# Disable Chrome even if globally configured on
claude --no-chrome --print "Process this without browser access"
```

## Notes

- Requires the Claude Code Chrome extension installed and active
- `--chrome` and `--no-chrome` are mutually exclusive; last one wins
- The Chrome integration is separate from the `WebFetch` and `WebSearch` built-in tools

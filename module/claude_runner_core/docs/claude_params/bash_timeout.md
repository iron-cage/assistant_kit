# bash_timeout

Default timeout for bash commands in milliseconds.

## Type

**Env** — environment variable

## Environment Variable

```
CLAUDE_CODE_BASH_TIMEOUT=<milliseconds>
```

## Default

`3600000` ms = 1 hour (in `claude_runner_core` builder)

Standard claude default: `120000` ms = 2 minutes

## Description

Sets the default timeout for each bash command Claude executes. If a bash command runs longer than this, Claude Code terminates it and returns an error.

The `claude_runner_core` builder defaults to 1 hour (3,600,000 ms) instead of the standard 2 minutes. The standard default is designed for interactive use where commands should complete quickly. In automation contexts, commands like `cargo test`, `docker build`, or `npm install` routinely exceed 2 minutes. The 1-hour default prevents premature timeouts in automated workflows.

For the maximum allowed timeout (cap on what individual commands can request), see `bash_max_timeout.md`.

## Builder API

```rust
use claude_runner_core::ClaudeCommand;

// Default: 1 hour
let cmd = ClaudeCommand::new();

// Custom timeout: 30 minutes
let cmd = ClaudeCommand::new()
  .with_bash_timeout_ms( 30 * 60 * 1000 );
```

Builder method: `with_bash_timeout_ms(timeout_ms: u32)` — sets `CLAUDE_CODE_BASH_TIMEOUT`.

## Examples

```bash
# Shell: 30-minute default timeout
CLAUDE_CODE_BASH_TIMEOUT=1800000 claude --print "Run the full CI pipeline"

# Short timeout for quick commands only
CLAUDE_CODE_BASH_TIMEOUT=10000 claude --print "Check git status"
```

## Notes

- `claude_runner_core` default (1hr) vs standard claude default (2min): prevents automation failures
- See `bash_max_timeout.md` for the upper bound cap
- Individual bash invocations can request up to `CLAUDE_CODE_BASH_MAX_TIMEOUT`
- Too-short timeouts cause `ClaudeCommand: bash command timed out` errors mid-automation

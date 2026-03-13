# bash_max_timeout

Maximum allowed timeout for bash commands in milliseconds — caps what individual commands can use.

## Type

**Env** — environment variable

## Environment Variable

```
CLAUDE_CODE_BASH_MAX_TIMEOUT=<milliseconds>
```

## Default

`7200000` ms = 2 hours (in `claude_runner_core` builder)

Standard claude default: `600000` ms = 10 minutes

## Description

Sets the upper bound on bash command timeouts. Individual bash commands can request up to this value, but no more. This is the cap for `CLAUDE_CODE_BASH_TIMEOUT`.

The `claude_runner_core` builder defaults to 2 hours (7,200,000 ms) instead of the standard 10 minutes. The 10-minute standard assumes interactive use. Automated workflows may run database migrations, full test suites, or long builds that exceed 10 minutes.

Relationship to `bash_timeout`:
- `CLAUDE_CODE_BASH_TIMEOUT`: the **default** per-command timeout
- `CLAUDE_CODE_BASH_MAX_TIMEOUT`: the **maximum** any command can use (ceiling)

Commands timing out with max timeout produce a hard error; increase this value for long-running automated tasks.

## Builder API

```rust
use claude_runner_core::ClaudeCommand;

// Default: 2 hours
let cmd = ClaudeCommand::new();

// Custom max: 4 hours for very long builds
let cmd = ClaudeCommand::new()
  .with_bash_max_timeout_ms( 4 * 60 * 60 * 1000 );
```

Builder method: `with_bash_max_timeout_ms(timeout_ms: u32)` — sets `CLAUDE_CODE_BASH_MAX_TIMEOUT`.

## Examples

```bash
# Allow up to 4 hours per command
CLAUDE_CODE_BASH_MAX_TIMEOUT=14400000 claude --print "Run full integration tests"

# Cap at 5 minutes for quick tasks
CLAUDE_CODE_BASH_MAX_TIMEOUT=300000 claude --print "Run unit tests only"
```

## Notes

- Must be ≥ `CLAUDE_CODE_BASH_TIMEOUT`; setting max lower than default creates a contradiction
- `claude_runner_core` default (2hr) vs standard claude default (10min): prevents automation failures
- Increase for workflows with long-running commands (builds, migrations, full test suites)

# log_level

Log verbosity level for Claude Code's internal logging.

## Type

**Env** — environment variable

## Environment Variable

```
CLAUDE_CODE_LOG_LEVEL=<level>
```

## Values

| Level | Description |
|-------|-------------|
| `error` | Only errors |
| `warn` | Errors and warnings |
| `info` | Standard operational messages (default) |
| `debug` | Detailed debug information |
| `trace` | Extremely verbose trace output |

## Default

`info` (inherits standard)

## Description

Controls the verbosity of Claude Code's internal log output. Higher levels produce more output; lower levels suppress all but critical messages.

This is distinct from `--verbose` (which controls Claude's response verbosity) and `--debug` (which enables CLI debug mode). `log_level` controls the internal application logger.

Log levels follow standard severity ordering: `error < warn < info < debug < trace`. Setting a level shows all messages at that level and above. E.g., `warn` shows warnings and errors but not info/debug/trace.

## Builder API

```rust
use claude_runner_core::{ ClaudeCommand, LogLevel };

// Default: Info
let cmd = ClaudeCommand::new();

// Debug level for troubleshooting
let cmd = ClaudeCommand::new()
  .with_log_level( LogLevel::Debug );
```

Builder method: `with_log_level(level: LogLevel)` — sets `CLAUDE_CODE_LOG_LEVEL`.

`LogLevel` enum values: `Error`, `Warn`, `Info`, `Debug`, `Trace`

The enum implements `Ord` for level comparison and `as_str()` for serialization.

## Examples

```bash
# Quiet operation (errors only)
CLAUDE_CODE_LOG_LEVEL=error claude --print "Silent run"

# Debug for troubleshooting
CLAUDE_CODE_LOG_LEVEL=debug claude --print "Trace this issue"

# Maximum verbosity
CLAUDE_CODE_LOG_LEVEL=trace claude --print "Ultra verbose"
```

## Notes

- Case-insensitive in the env var (both `info` and `Info` are accepted)
- The `LogLevel` Rust enum uses PascalCase (`Info`, `Debug`), but the env var accepts lowercase
- For tool-level debug (API calls, bash commands), `--debug` / `--debug-file` is more useful

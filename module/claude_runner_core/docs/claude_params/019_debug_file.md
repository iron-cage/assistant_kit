# debug_file

Write debug logs to a specific file path instead of stderr.

## Type

**CLI** — string value (file path)

## Syntax

```
claude --debug-file <path>
```

## Default

None (debug output goes to stderr when `--debug` is set)

## Description

Redirects debug output to a specified file path. Setting `--debug-file` implicitly enables debug mode — `--debug` is not required separately.

Useful for:
- Capturing debug logs from non-interactive sessions without polluting stderr
- Persistent logs for post-session analysis
- CI pipelines where debug info should be captured as build artifacts
- Comparing debug output across multiple runs

The file is created if it doesn't exist; existing files are overwritten.

## Builder API

Use `with_debug_file()` — Accepts a file path string.

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_debug_file( "/tmp/claude-debug.log" )
  .with_message( "Run with debug log" );
```

## Examples

```bash
# Capture debug to file (--debug not needed)
claude --debug-file /tmp/claude.log --print "Analyze this"

# With category filter
claude --debug api --debug-file /var/log/claude-api.log --print "API trace"

# Timestamped log file
claude --debug-file "/tmp/claude-$(date +%s).log" "Debug session"
```

## Notes

- Implicitly enables debug mode; `--debug` flag is optional when `--debug-file` is set
- File is overwritten on each run (not appended)
- The path must be writable; a directory path will cause an error

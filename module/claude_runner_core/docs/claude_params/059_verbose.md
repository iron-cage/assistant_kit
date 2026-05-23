# verbose

Override verbose output mode for the session.

## Type

**CLI** — boolean flag

## Syntax

```
claude --verbose
```

## Default

off (inherits from config)

## Description

Forces verbose output mode on for the current session, overriding whatever the config file specifies. In verbose mode, Claude Code emits additional diagnostic information including tool call details, token usage, and internal processing steps.

This is a session-level override — it does not persist to config.

## Builder API

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_verbose( true );
```

Builder method: `with_verbose(verbose: bool)` — adds `--verbose` to args when `true`, adds nothing when `false`.

## Examples

```bash
# Enable verbose for one session
claude --verbose "Debug this issue"

# Verbose in non-interactive mode
claude --print --verbose "What tools did you use?"
```

## Notes

- Does not correspond to a `CLAUDE_CODE_*` env var; set via CLI flag only
- For logging verbosity (error/warn/info/debug/trace), see `log_level.md` instead
- Config-level verbose setting is in `~/.claude/settings.json` under `verboseMode`

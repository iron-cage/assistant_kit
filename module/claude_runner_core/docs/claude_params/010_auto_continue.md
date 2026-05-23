# auto_continue

Enable automatic continuation without manual prompts.

## Type

**Env** — environment variable

## Environment Variable

```
CLAUDE_CODE_AUTO_CONTINUE=<bool>
```

Values: `true` / `false`

## Default

`true` (in `claude_runner_core` builder)

Standard claude default: `false`

## Description

When enabled, Claude Code automatically continues executing without prompting the user for confirmation at decision points. This is essential for unattended automation.

The `claude_runner_core` builder defaults to `true` (auto-continue enabled), unlike the standard `false`. In interactive use, `false` is safe — users see and approve each step. In automation, `false` blocks the pipeline waiting for input that never arrives.

With `auto_continue = true`:
- Claude proceeds through multi-step tasks without pausing for confirmation
- Long-running autonomous tasks complete without human intervention
- Combined with `dangerously_skip_permissions` for fully autonomous execution

## Builder API

```rust
use claude_runner_core::ClaudeCommand;

// Default: auto-continue enabled
let cmd = ClaudeCommand::new();

// Explicitly disable for controlled automation
let cmd = ClaudeCommand::new()
  .with_auto_continue( false );
```

Builder method: `with_auto_continue(auto_continue: bool)` — sets `CLAUDE_CODE_AUTO_CONTINUE`.

## Examples

```bash
# Enable for automation (already default in builder)
CLAUDE_CODE_AUTO_CONTINUE=true claude --print "Run the full migration"

# Disable for controlled step-by-step (interactive only)
CLAUDE_CODE_AUTO_CONTINUE=false claude "Walk me through each change"
```

## Notes

- `claude_runner_core` default (`true`) vs standard claude default (`false`): enables automation without manual intervention
- In `--print` mode, `auto_continue=false` may cause the session to hang waiting for input
- For fully automated pipelines, combine with `with_skip_permissions(true)`

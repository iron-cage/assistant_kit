# permission_mode

Fine-grained permission mode controlling how Claude Code handles tool approval.

## Type

**CLI** — enum value

## Syntax

```
claude --permission-mode <mode>
```

## Values

| Mode | Description |
|------|-------------|
| `default` | Standard behavior: ask for permission on sensitive operations |
| `acceptEdits` | Auto-accept file edits; still ask for shell commands |
| `bypassPermissions` | Bypass all permissions (equivalent to `--dangerously-skip-permissions`) |
| `dontAsk` | Never ask for permission (similar to bypass) |
| `plan` | Plan mode: Claude describes what it will do without executing |
| `auto` | Automatic mode: Claude decides when to ask vs proceed |

## Default

`default`

## Description

Provides granular control over which tool calls require user approval. More expressive than the binary `--dangerously-skip-permissions` flag.

**Use cases by mode**:
- `default`: Interactive developer sessions where you want to review changes
- `acceptEdits`: Automation where file edits are safe but shell commands need review
- `bypassPermissions` / `dontAsk`: Fully automated pipelines (same safety caveats as `--dangerously-skip-permissions`)
- `plan`: Dry-run or review mode — see what Claude would do without doing it
- `auto`: Balanced automation where Claude exercises judgment

## Builder API

Use `with_permission_mode()` — Accepts a `PermissionMode` enum value (`Default`, `AcceptEdits`, or `BypassPermissions`).

```rust
use claude_runner_core::{ ClaudeCommand, PermissionMode };

let cmd = ClaudeCommand::new()
  .with_permission_mode( PermissionMode::AcceptEdits )
  .with_message( "Auto-accept edit operations" );
```

## Examples

```bash
# Auto-accept edits, ask for shell
claude --permission-mode acceptEdits --print "Fix all formatting issues"

# Plan mode: review without executing
claude --permission-mode plan --print "Migrate the database schema"

# Full bypass for CI
claude --permission-mode bypassPermissions --print "Run the test suite and fix failures"
```

## Notes

- `bypassPermissions` and `dontAsk` have the same security implications as `--dangerously-skip-permissions`
- `plan` mode is useful for reviewing proposed changes before committing to execution
- Mode applies to all tool calls in the session

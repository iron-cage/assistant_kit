# sandbox_mode

Enable or disable sandbox mode for process execution isolation.

## Type

**Env** — environment variable

## Environment Variable

```
CLAUDE_CODE_SANDBOX_MODE=<bool>
```

Values: `true` / `false`

## Default

`true` (inherits standard)

## Description

Controls whether Claude Code's subprocess execution is sandboxed. When enabled, bash commands and other process invocations run within a sandboxed environment that restricts certain system calls and access patterns.

Sandbox mode provides an additional isolation layer on top of the tool permission system. It limits what processes spawned by Claude can do even if permissions are granted.

Setting to `false` disables sandbox restrictions, giving spawned processes full system access. This may be needed for:
- Tools that require elevated system access (e.g., package managers, build tools)
- Environments already running in a container (double-sandboxing is unnecessary)
- Debugging sandbox-related failures

## Builder API

```rust
use claude_runner_core::ClaudeCommand;

// Default: true (inherits standard)
let cmd = ClaudeCommand::new();

// Disable sandbox for full system access
let cmd = ClaudeCommand::new()
  .with_sandbox_mode( false );
```

Builder method: `with_sandbox_mode(sandbox: bool)` — sets `CLAUDE_CODE_SANDBOX_MODE`.

## Examples

```bash
# Disable sandbox (e.g., inside Docker where it's already isolated)
CLAUDE_CODE_SANDBOX_MODE=false claude --print "Run npm install"

# Explicit sandbox enable
CLAUDE_CODE_SANDBOX_MODE=true claude --print "Run in restricted environment"
```

## Notes

- When running in Docker/containers, `false` is often appropriate since the container itself provides isolation
- Sandbox mode availability depends on the OS and Claude Code version (may be no-op on some platforms)
- Security tradeoff: sandbox prevents escalation but may break tools requiring system-level access

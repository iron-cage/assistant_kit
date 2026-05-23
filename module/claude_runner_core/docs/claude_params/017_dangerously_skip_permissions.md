# dangerously_skip_permissions

Bypass all permission checks. Enables fully autonomous execution without approval prompts.

## Type

**CLI** — boolean flag

## Syntax

```
claude --dangerously-skip-permissions
```

## Default

off (in the `claude` binary itself)

**Note**: `claude_runner_core`'s `ClaudeCommand` builder sets `skip_permissions = false` by default, but the `clr` wrapper CLI adds `--dangerously-skip-permissions` automatically. Use `--no-skip-permissions` in `clr` to opt out.

## Description

When set, Claude Code bypasses all tool permission prompts and executes tools immediately without waiting for user approval. This includes file writes, shell commands, network requests, and any other potentially destructive operations.

The name "dangerously" is intentional — this flag removes a critical safety mechanism.

**Recommended use cases**: sandboxed CI/CD environments, Docker containers with no internet access, automated testing pipelines where the execution environment is controlled.

**Do not use**: on developer machines, in production environments with network access, or when processing untrusted input.

### Difference from `--allow-dangerously-skip-permissions`

- `--dangerously-skip-permissions`: Permissions are skipped unconditionally; Claude never asks.
- `--allow-dangerously-skip-permissions`: Makes skip-permissions available as an option Claude can choose to use, without it being the forced default.

## Builder API

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_skip_permissions( true );
```

Builder method: `with_skip_permissions(skip: bool)` — adds `--dangerously-skip-permissions` to args when `true`.

## Examples

```bash
# In a sandboxed Docker container
claude --dangerously-skip-permissions --print "Fix all lint errors in this repo"

# CI/CD automation
claude --dangerously-skip-permissions -c "Apply the suggested changes"
```

## Notes

- Only recommended for sandboxes with no internet access (per official claude docs)
- `clr` wrapper adds this flag by default; use `--no-skip-permissions` to disable
- Permission checks are entirely absent — any tool Claude calls executes immediately

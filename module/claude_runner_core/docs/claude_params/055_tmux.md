# tmux

Create a tmux session for the worktree.

## Type

**CLI** — boolean flag (with optional variant)

## Syntax

```
claude --worktree [name] --tmux
claude --worktree [name] --tmux=classic
```

## Default

off

## Description

When used with `--worktree`, opens the new worktree session in a tmux pane rather than the current terminal.

Behavior depends on the terminal environment:
- **iTerm2**: Uses native iTerm2 panes for a native macOS experience
- **Other terminals**: Uses standard tmux panes (`--tmux=classic` forces this)

The `--tmux=classic` variant always uses traditional tmux behavior, ignoring iTerm2 native panes.

Requires:
- `--worktree` to be set (can only be used with worktrees)
- `tmux` to be installed and accessible in PATH

## Builder API

Use `with_tmux()` — Boolean flag: creates a tmux session for the worktree.

```rust
use claude_runner_core::ClaudeCommand;

let cmd = ClaudeCommand::new()
  .with_worktree( Some( "feature-branch" ) )
  .with_tmux( true )
  .with_message( "Run in tmux session" );
```

## Examples

```bash
# Open worktree in tmux pane (auto-detects iTerm2)
claude --worktree feature-auth --tmux

# Force classic tmux behavior
claude --worktree feature-auth --tmux=classic

# Multi-window workflow
claude --worktree task-a --tmux &
claude --worktree task-b --tmux &
```

## Notes

- Cannot be used without `--worktree`
- The tmux pane is attached to the worktree's directory automatically
- Useful for running multiple Claude sessions in parallel, each in its own pane

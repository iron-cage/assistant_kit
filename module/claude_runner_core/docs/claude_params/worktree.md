# worktree

Create a new git worktree for the session, optionally with a specified name.

## Type

**CLI** — optional string value

## Syntax

```
claude --worktree [name]
claude -w [name]
```

## Default

None (no worktree created)

## Description

Creates a new git worktree before starting the Claude session. Claude then operates within that worktree. This allows Claude to make changes in an isolated branch without touching the main working tree.

The worktree is created from the current HEAD. If a name is provided, it's used as the worktree directory name. If omitted, Claude generates a name.

Use with `--tmux` to open the worktree session in a tmux pane.

The worktree feature requires the current directory to be a git repository.

## Builder API

Use `with_worktree()` — Optional-value: `Some(name)` adds `--worktree name`, `None` adds `--worktree` without a name.

```rust
use claude_runner_core::ClaudeCommand;

// Create named worktree
let cmd = ClaudeCommand::new()
  .with_worktree( Some( "my-feature" ) )
  .with_message( "Work in named worktree" );

// Create anonymous worktree
let cmd = ClaudeCommand::new()
  .with_worktree( None::<String> )
  .with_message( "Work in auto-named worktree" );
```

## Examples

```bash
# Create unnamed worktree
claude --worktree --print "Implement the new feature"

# Create named worktree
claude --worktree feature-login --print "Add login functionality"

# Worktree with tmux pane
claude --worktree my-feature --tmux
```

## Notes

- Requires git repository with at least one commit
- The worktree is a separate checkout — changes are isolated until merged
- Worktree name becomes the branch name for the isolated work
- After the session, the worktree must be manually removed with `git worktree remove`

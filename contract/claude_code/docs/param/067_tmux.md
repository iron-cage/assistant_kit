# tmux

Creates a tmux session for the worktree when used with `--worktree`.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--tmux` |
| Env Var | — |
| Config Key | — |

### Type

bool

### Default

`off`

### Since

pre-v1.0 (unverified)

### Description

Creates a tmux session for the worktree when used with `--worktree`. In iTerm2, uses native panes; otherwise uses traditional tmux. Append `=classic` (`--tmux=classic`) to force traditional tmux behaviour even in iTerm2. Requires `--worktree` — has no effect without it.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [073_worktree.md](073_worktree.md) | Worktree flag (required for --tmux) |
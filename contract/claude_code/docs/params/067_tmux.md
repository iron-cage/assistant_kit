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

### Description

Creates a tmux session for the worktree when used with `--worktree`. In iTerm2, uses native panes; otherwise uses traditional tmux. Append `=classic` (`--tmux=classic`) to force traditional tmux behaviour even in iTerm2. Requires `--worktree` — has no effect without it.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
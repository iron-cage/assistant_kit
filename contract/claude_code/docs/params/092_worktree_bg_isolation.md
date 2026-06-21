# worktree_bg_isolation

Controls whether worktree sessions run in background isolation.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | — |
| Config Key | `worktree.bgIsolation` |

### Type

bool

### Default

false

### Since

v2.1.143

### Description

When enabled, git worktree sessions (`--worktree`) run in background isolation
mode. This keeps worktree sessions separate from the main terminal, allowing
the user to continue working while the worktree session runs independently.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [073_worktree.md](073_worktree.md) | Worktree session creation |
| doc | [067_tmux.md](067_tmux.md) | Tmux session for worktree |

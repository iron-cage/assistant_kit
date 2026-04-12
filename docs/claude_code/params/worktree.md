# worktree

Creates a new git worktree for the session, scoping file operations to it.

### Forms

| | Value |
|-|-------|
| CLI Flag | `-w` / `--worktree [name]` |
| Env Var | — |
| Config Key | — |

### Type

string? (optional worktree name)

### Default

—

### Description

Creates a new git worktree for the session. With no name argument, an auto-generated name is used. The worktree is created as a sibling directory to the current repo. Claude's file operations are scoped to the new worktree, keeping the main working tree clean. Often paired with `--tmux` to open the worktree in a dedicated terminal pane.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
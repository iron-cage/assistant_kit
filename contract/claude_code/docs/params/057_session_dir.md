# session_dir

Overrides the directory where session `.jsonl` files are stored for the current invocation.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_SESSION_DIR` |
| Config Key | — |

### Type

path

### Default

`auto` (`~/.claude/projects/{encoded-path}/`)

### Description

Overrides the directory where session `.jsonl` files are stored for the current invocation. When set, Claude reads and writes session files from this path instead of the default project-encoded directory under `~/.claude/projects/`. Useful for redirecting session storage to a custom location in CI or multi-user environments.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
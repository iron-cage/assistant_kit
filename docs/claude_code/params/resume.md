# resume

Resumes a previous conversation by session ID or interactive picker.

### Forms

| | Value |
|-|-------|
| CLI Flag | `-r` / `--resume [id]` |
| Env Var | — |
| Config Key | — |

### Type

string? (optional session ID)

### Default

—

### Description

Resumes a previous conversation. With no argument, opens an interactive picker listing recent sessions. With a session ID argument, resumes that specific session directly. The session ID is the UUID from the `.jsonl` filename in `~/.claude/projects/`. See also `--fork-session` to resume without overwriting the original.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
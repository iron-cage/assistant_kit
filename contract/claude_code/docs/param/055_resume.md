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

### Since

pre-v1.0 (unverified)

### Description

Resumes a previous conversation. With no argument, opens an interactive picker listing recent sessions. With a session ID argument, resumes that specific session directly. The session ID is the UUID from the `.jsonl` filename in `~/.claude/projects/`. See also `--fork-session` to resume without overwriting the original.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [017_continue.md](017_continue.md) | Continue most recent session (simpler alternative) |
| doc | [029_fork_session.md](029_fork_session.md) | Resume without modifying original |
| doc | [043_no_session_persistence.md](043_no_session_persistence.md) | Disable session storage entirely |
| doc | [058_session_id.md](058_session_id.md) | Session ID identifier |
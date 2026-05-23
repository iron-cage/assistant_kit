# fork_session

Creates a new session ID when resuming, branching from the past checkpoint without modifying the original.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--fork-session` |
| Env Var | — |
| Config Key | — |

### Type

bool

### Default

`off`

### Description

When resuming a session (via `--resume` or `--continue`), creates a new session ID rather than appending to the original. The resumed history is copied into a new `.jsonl` file, preserving the original session unchanged. Use this to branch from a past checkpoint without affecting the original conversation.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
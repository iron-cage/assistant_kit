# no_session_persistence

Disables writing the session to disk so it cannot be resumed later.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--no-session-persistence` |
| Env Var | — |
| Config Key | — |

### Type

bool

### Default

`off`

### Description

Disables writing the session to disk. No `.jsonl` file is created and the session cannot be resumed later. Useful for ephemeral invocations where session history is unwanted (e.g. one-shot CI queries, privacy-sensitive contexts). Only works with `--print` mode.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
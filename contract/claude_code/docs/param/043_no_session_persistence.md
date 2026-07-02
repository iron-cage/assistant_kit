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

### Since

pre-v1.0 (unverified)

### Description

Disables writing the session to disk. No `.jsonl` file is created and the session cannot be resumed later. Useful for ephemeral invocations where session history is unwanted (e.g. one-shot CI queries, privacy-sensitive contexts). Only works with `--print` mode.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [055_resume.md](055_resume.md) | Resume sessions (disabled by this flag) |
| doc | [017_continue.md](017_continue.md) | Continue sessions (disabled by this flag) |
| doc | [051_print.md](051_print.md) | Print mode (required for this flag to apply) |
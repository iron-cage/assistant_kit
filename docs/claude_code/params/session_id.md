# session_id

Assigns a specific UUID to the current session instead of generating one automatically.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--session-id <uuid>` |
| Env Var | — |
| Config Key | — |

### Type

uuid

### Default

auto-generated

### Description

Assigns a specific UUID to the current session instead of generating one automatically. The UUID must be a valid v4 UUID. Useful for reproducible automation where session identity must be deterministic (e.g. linking Claude invocations to external tracking systems). If the UUID already exists as a session file, behaviour depends on other flags (`--resume`, `--fork-session`).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
# continue

Explicitly continue the most recent conversation in the current directory.

### Forms

| | Value |
|-|-------|
| CLI Flag | `-c` / `--continue` |
| Env Var | — |
| Config Key | — |

### Type

bool

### Default

`off` (continuation is the default behaviour without this flag, but the flag makes it explicit)

### Description

Explicitly continue the most recent conversation in the current directory. This is actually the default behaviour of `claude` — the flag exists as an explicit alias. Use `--no-session-persistence` or start a fresh session to break continuation. See also `--resume` for continuing by session ID.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
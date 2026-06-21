# tmpdir

Overrides the temporary directory used by Claude Code for internal operations.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_TMPDIR` |
| Config Key | — |

### Type

path

### Default

System temp directory (`/tmp` on Linux, `$TMPDIR` on macOS)

### Since

v2.1.161

### Description

Sets a custom temporary directory for Claude Code's internal temporary files,
scratch space, and intermediate results. Useful when the system temp directory
has size constraints or is on a slow filesystem.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |

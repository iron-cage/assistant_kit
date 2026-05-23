# debug_file

Writes debug log output to the specified file path.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--debug-file <path>` |
| Env Var | — |
| Config Key | — |

### Type

path

### Default

—

### Description

Writes debug log output to the specified file path in addition to (or instead of) stderr. Implicitly enables debug mode — no need to also pass `--debug`. Useful for capturing verbose diagnostic output in CI or long-running sessions without polluting the terminal. The file is created or appended to if it already exists.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
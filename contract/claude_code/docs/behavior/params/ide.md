# ide

Automatically connects Claude to an IDE on startup if exactly one is available.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--ide` |
| Env Var | — |
| Config Key | — |

### Type

bool

### Default

`off`

### Description

Automatically connects Claude to an IDE on startup, if exactly one valid IDE integration is available (e.g. VS Code with the Claude extension). When multiple IDEs are available, no automatic connection is made — the user must choose manually. Enables IDE-specific tools and context sharing between the terminal and the editor.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
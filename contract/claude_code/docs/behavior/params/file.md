# file

Downloads Anthropic Files API resources at startup and makes them available in the session.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--file <specs...>` |
| Env Var | — |
| Config Key | — |

### Type

string[] (space-separated `file_id:relative_path` specs)

### Default

—

### Description

Downloads Anthropic Files API resources at startup and makes them available to Claude in the session. Each spec is `file_id:relative_path`, e.g. `--file file_abc123:context.txt`. The file is fetched and written to the specified relative path before Claude processes the first message. Useful for injecting large context documents without embedding them in the prompt.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
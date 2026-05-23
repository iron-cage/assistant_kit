# tools

Overrides the full set of available tools for the session.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--tools <tools...>` |
| Env Var | — |
| Config Key | — |

### Type

string[] (space or comma separated)

### Default

`default`

### Description

Overrides the full set of available tools. `default` (the default value) enables all built-in tools. `""` (empty string) disables every tool — Claude cannot read files, run bash, or use any other tool. Named tools can be listed to use exactly that set. This is a coarser override than `--allowed-tools`/`--disallowed-tools`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
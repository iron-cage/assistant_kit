# disallowed_tools

Blocks specific tools from being available for the session.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--disallowed-tools <tools...>` |
| Env Var | — |
| Config Key | `disallowedTools` |

### Type

string[] (space or comma separated)

### Default

none denied

### Since

pre-v1.0 (unverified)

### Description

Blocks specific tools from being available for the session. Accepts the same tool name format as `--allowed-tools`. The listed tools are removed from the available set; all others remain. Useful for targeted disabling without enumerating all permitted tools. When both `--allowed-tools` and `--disallowed-tools` are present, the disallowed list is subtracted from the allowed set.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [006_allowed_tools.md](006_allowed_tools.md) | Tool allowlist (complement) |
| doc | [068_tools.md](068_tools.md) | Full tool override (coarser control) |
# enabled_plugins

The active plugin registry stored in settings, read at startup to initialise plugins.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | — |
| Config Key | `enabledPlugins` (in `~/.claude/settings.json`) |

### Type

object

### Default

`{}`

### Description

The active plugin registry stored in settings. Keys are plugin identifiers; values are plugin configuration objects. Claude Code reads this at startup to determine which plugins to initialise. For session-scoped plugin loading without persisting to settings, use `--plugin-dir` instead. Managed by Claude Code's plugin system; not intended for direct manual editing.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
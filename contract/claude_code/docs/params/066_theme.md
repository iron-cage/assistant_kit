# theme

Sets the UI color theme for Claude Code's terminal interface.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | — |
| Config Key | `theme` (in `~/.claude/settings.json`) |

### Type

string

### Default

`"dark"`

### Description

Sets the UI color theme for Claude Code's terminal interface. The value is a theme name string. The binary ships with at least `"dark"` and `"light"` built-in themes. This setting is read once at startup from the settings file; it cannot be overridden per-invocation via CLI or env var. Modify via `cm .settings.set key::theme value::light` or by editing `~/.claude/settings.json` directly.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
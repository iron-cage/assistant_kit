# auto_updates

Controls whether Claude Code automatically updates its binary on startup.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | — |
| Config Key | `autoUpdates` (in `~/.claude/settings.json`) |

### Type

bool

### Default

`true`

### Description

Controls whether Claude Code automatically updates its binary on startup. When true (the default), the binary checks for and installs newer versions on launch. Set to false to pin to a specific version — typically managed by `cm .version.install` along with the `preferredVersionSpec` and `DISABLE_AUTOUPDATER` env override. Read once at startup; cannot be overridden per-invocation.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
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

### Since

pre-v1.0 (unverified)

### Description

Controls whether Claude Code automatically updates its binary on startup. When true (the default), the binary checks for and installs newer versions on launch. Set to false to pin to a specific version — typically managed by `cm .version.install` along with the `preferredVersionSpec` and `DISABLE_AUTOUPDATER` env override. Read once at startup; cannot be overridden per-invocation.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [099_disable_autoupdater.md](099_disable_autoupdater.md) | Env var to disable background auto-updates |
| doc | [121_auto_updates_channel.md](121_auto_updates_channel.md) | Release channel selector (latest/stable) |
| doc | [../subcommand/004_doctor.md](../subcommand/004_doctor.md) | Auto-updater health check |
| doc | [../subcommand/009_update.md](../subcommand/009_update.md) | Manual update subcommand |
| doc | [../../../../module/claude_version/docs/pattern/001_version_lock.md](../../../../module/claude_version/docs/pattern/001_version_lock.md) | This repo's own tooling that sets this key as Layer 1 of its 8-layer pin |
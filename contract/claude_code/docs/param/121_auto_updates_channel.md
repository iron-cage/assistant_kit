# Parameter: auto_updates_channel

### Forms

| Form | Value |
|------|-------|
| Config Key | `autoUpdatesChannel` (user, project, and local `settings.json`; enforceable via managed settings) |

### Type

string (`"latest"` \| `"stable"`)

### Default

`"latest"`

### Description

Selects the release channel Claude Code follows for background auto-updates and `claude update`. `"latest"` (the default) delivers new features as soon as they ship. `"stable"` tracks a version that is typically about one week old and skips releases with major regressions. Configurable via `/config` → **Auto-update channel**, or directly as a `settings.json` key; enforceable organization-wide through managed settings. Switching from `"latest"` to `"stable"` via `/config` prompts the user to stay on the current version (setting `minimumVersion`) or allow the downgrade. To stop updates entirely rather than choosing a channel, use `DISABLE_UPDATES` (`DISABLE_AUTOUPDATER` only stops the background check; manual `claude update` still works). Homebrew installs select a channel by cask name instead of this key (`claude-code` tracks stable, `claude-code@latest` tracks latest).

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [011_auto_updates.md](011_auto_updates.md) | Boolean master switch for auto-updates |
| doc | [099_disable_autoupdater.md](099_disable_autoupdater.md) | Env var to disable background auto-updates entirely |
| doc | [119_disable_updates.md](119_disable_updates.md) | Env var to block all updates (auto + manual) |
| doc | [122_minimum_version.md](122_minimum_version.md) | Version floor set automatically on channel downgrade |

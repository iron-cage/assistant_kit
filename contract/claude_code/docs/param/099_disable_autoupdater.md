# Parameter: disable_autoupdater

### Forms

| Form | Value |
|------|-------|
| Env Var | `DISABLE_AUTOUPDATER` |

### Type

boolean (presence-activated)

### Default

Not set (auto-updater enabled)

### Description

Disables background auto-updates. Manual updates via `claude update` still work.
For blocking all updates (auto + manual), use `DISABLE_UPDATES` instead.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [011_auto_updates.md](011_auto_updates.md) | Boolean master switch for auto-updates |
| doc | [119_disable_updates.md](119_disable_updates.md) | Block all updates |
| doc | [120_disable_upgrade_command.md](120_disable_upgrade_command.md) | Hide /upgrade slash command |
| doc | [121_auto_updates_channel.md](121_auto_updates_channel.md) | Release channel selector (latest/stable) |
| doc | [125_package_manager_auto_update.md](125_package_manager_auto_update.md) | Homebrew/WinGet auto-upgrade opt-in |
| doc | [126_disable_nonessential_traffic.md](126_disable_nonessential_traffic.md) | Combined opt-out including this var |
| doc | [../subcommand/009_update.md](../subcommand/009_update.md) | Update subcommand |
| doc | [../pattern/001_version_pinning.md](../pattern/001_version_pinning.md) | Synthesis: full version-pinning landscape |

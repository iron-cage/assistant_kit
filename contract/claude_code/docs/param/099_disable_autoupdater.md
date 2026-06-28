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
| doc | [119_disable_updates.md](119_disable_updates.md) | Block all updates |
| doc | [120_disable_upgrade_command.md](120_disable_upgrade_command.md) | Hide /upgrade slash command |
| doc | [../subcommand/009_update.md](../subcommand/009_update.md) | Update subcommand |

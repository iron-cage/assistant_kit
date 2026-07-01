# Parameter: disable_updates

### Forms

| Form | Value |
|------|-------|
| Env Var | `DISABLE_UPDATES` |

### Type

boolean (presence-activated)

### Default

Not set (updates enabled)

### Description

Blocks ALL updates — both background auto-updates and manual `claude update`.
Stronger than `DISABLE_AUTOUPDATER` which only blocks background updates.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [099_disable_autoupdater.md](099_disable_autoupdater.md) | Disable background updates only |
| doc | [120_disable_upgrade_command.md](120_disable_upgrade_command.md) | Hide upgrade command |
| doc | [../subcommand/009_update.md](../subcommand/009_update.md) | Update subcommand |

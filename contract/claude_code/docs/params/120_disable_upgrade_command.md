# Parameter: disable_upgrade_command

### Forms

| Form | Value |
|------|-------|
| Env Var | `DISABLE_UPGRADE_COMMAND` |

### Type

boolean (presence-activated)

### Default

Not set (`/upgrade` visible)

### Description

Hides the `/upgrade` slash command from the interactive session. The `claude update`
CLI subcommand remains available unless `DISABLE_UPDATES` is also set.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [119_disable_updates.md](119_disable_updates.md) | Block all updates |
| doc | [../subcommand/009_update.md](../subcommand/009_update.md) | Update subcommand |

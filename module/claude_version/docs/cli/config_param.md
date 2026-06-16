# Config Parameters

### Scope

- **Purpose**: Config file parameter reference for the clv CLI.
- **Responsibility**: Document whether clv reads a config file and what parameters it supports.
- **In Scope**: Any persistent configuration files read by clv at startup.
- **Out of Scope**: CLI parameter reference (→ `005_params.md`), environment variable reference (→ `env_param.md`).

### Config File

clv has no config file. All behavior is controlled via CLI parameters passed at invocation time. There are no persistent configuration files specific to clv itself.

**Settings file:** `~/.claude/settings.json` is the *target* that clv reads and writes via `.settings.get` / `.settings.set` / `.settings.show`. It is not a config file for clv — it is the data store operated on by clv.

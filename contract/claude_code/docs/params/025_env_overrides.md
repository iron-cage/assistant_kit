# env_overrides

A JSON object of environment variables injected into the Claude Code process at every startup.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | — |
| Config Key | `env` (in `~/.claude/settings.json`) |

### Type

object (string→string map)

### Default

`{}`

### Since

pre-v1.0 (unverified)

### Description

A JSON object of environment variables injected into the Claude Code process at every startup, before any other env resolution. Entries override the shell environment for the process. Commonly used to set `DISABLE_AUTOUPDATER=1` as a permanent auto-update lock. Managed by `cm .settings.set key::env.VAR_NAME value::VAL` — the `env` sub-object is preserved as raw JSON by the settings writer.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [060_settings.md](060_settings.md) | Settings file (contains this object) |
| doc | [059_setting_sources.md](059_setting_sources.md) | Which settings layers are loaded |
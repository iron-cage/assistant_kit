# settings

Loads additional settings from a JSON file path or inline JSON string.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--settings <file-or-json>` |
| Env Var | — |
| Config Key | — |

### Type

string (file path or JSON string)

### Default

—

### Since

pre-v1.0 (unverified)

### Description

Loads additional settings from a JSON file path or an inline JSON string, supplementing the default settings loaded from `~/.claude/settings.json`. Settings from this source are merged with (not replacing) the default config. Useful for per-invocation or per-project settings overrides without modifying the global config file.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [059_setting_sources.md](059_setting_sources.md) | Filter which setting layers are loaded |
| doc | [025_env_overrides.md](025_env_overrides.md) | Per-session env overrides in settings |
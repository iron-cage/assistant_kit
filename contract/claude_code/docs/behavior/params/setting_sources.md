# setting_sources

Filters which setting source layers are loaded at startup.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--setting-sources <sources>` |
| Env Var | — |
| Config Key | — |

### Type

string (comma-separated) — valid values: `user` `project` `local` (any combination)

### Default

all

### Description

Filters which setting source layers are loaded at startup. `user` = `~/.claude/settings.json`. `project` = `.claude/settings.json` in the project root. `local` = `.claude/settings.local.json`. Specify a comma-separated subset to load only those layers, ignoring others. Useful for isolating config layers during debugging or in environments where some layers are absent.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
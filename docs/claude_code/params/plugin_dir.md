# plugin_dir

Loads Claude Code plugins from the specified directories for this session only.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--plugin-dir <paths...>` |
| Env Var | — |
| Config Key | — |

### Type

path[] (space-separated)

### Default

—

### Description

Loads Claude Code plugins from the specified directories for this session only. Plugins extend Claude's capabilities with additional tools or behaviours. The directories are scanned for plugin manifests at startup. This is a session-scoped override; for persistent plugin registration use the `enabledPlugins` settings config key instead.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
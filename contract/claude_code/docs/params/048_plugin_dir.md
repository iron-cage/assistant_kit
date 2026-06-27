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

### Since

pre-v1.0 (unverified)

### Description

Loads Claude Code plugins from the specified directories for this session only. Plugins extend Claude's capabilities with additional tools or behaviours. The directories are scanned for plugin manifests at startup. This is a session-scoped override; for persistent plugin registration use the `enabledPlugins` settings config key instead.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [024_enabled_plugins.md](024_enabled_plugins.md) | Persistent plugin registry (alternative) |
| doc | [088_plugin_prefer_https.md](088_plugin_prefer_https.md) | Plugin HTTPS preference |
| doc | [../subcommand/007_plugin.md](../subcommand/007_plugin.md) | Plugin management subcommand |
# Subcommand: plugin

Manage Claude Code plugins.

### Usage

```
claude plugin|plugins [command]
```

### Sub-subcommands

| Command | Description |
|---------|-------------|
| `install\|i [options] <plugin>` | Install a plugin from available marketplaces (`plugin@marketplace` for specific) |
| `uninstall\|remove [options] <plugin>` | Uninstall an installed plugin |
| `update [options] <plugin>` | Update a plugin to latest version (restart required) |
| `enable [options] <plugin>` | Enable a disabled plugin |
| `disable [options] [plugin]` | Disable an enabled plugin |
| `list [options]` | List installed plugins |
| `marketplace` | Manage Claude Code plugin marketplaces |
| `validate [options] <path>` | Validate a plugin or marketplace manifest |

### Description

Full plugin lifecycle management. Plugins extend Claude Code with additional
tools, MCP servers, and capabilities. They can be installed from marketplaces,
enabled/disabled individually, and validated for correctness.

The `marketplace` sub-subcommand manages the list of plugin sources. The
`validate` sub-subcommand checks a local plugin manifest for structural
correctness.

Alias: `claude plugins` works identically to `claude plugin`.

### Since

v2.0.12 (2025-10-09)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master subcommand table |
| doc | [../params/024_enabled_plugins.md](../params/024_enabled_plugins.md) | `enabledPlugins` config key |
| doc | [../params/048_plugin_dir.md](../params/048_plugin_dir.md) | `--plugin-dir` parameter |
| doc | [../params/088_plugin_prefer_https.md](../params/088_plugin_prefer_https.md) | Plugin HTTPS preference |

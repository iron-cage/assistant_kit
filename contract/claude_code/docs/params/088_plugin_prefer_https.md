# plugin_prefer_https

Forces plugin communication to use HTTPS transport.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_PLUGIN_PREFER_HTTPS` |
| Config Key | — |

### Type

bool

### Default

false

### Since

v2.1.141

### Description

When set to true, Claude Code prefers HTTPS transport for plugin communication
instead of the default stdio or local socket transport. Useful in environments
where plugins run as remote services.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [048_plugin_dir.md](048_plugin_dir.md) | Plugin directory configuration |
| doc | [024_enabled_plugins.md](024_enabled_plugins.md) | Active plugin registry |

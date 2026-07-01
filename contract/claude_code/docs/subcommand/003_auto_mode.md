# Subcommand: auto-mode

Inspect auto mode classifier configuration.

### Usage

```
claude auto-mode [command]
```

### Sub-subcommands

| Command | Description |
|---------|-------------|
| `config` | Print effective auto mode config as JSON (user settings with defaults) |
| `defaults` | Print default auto mode environment, allow, and deny rules as JSON |

### Description

Inspects the configuration of the auto-mode permission classifier. The `config`
sub-subcommand shows the effective merged configuration (user overrides + defaults).
The `defaults` sub-subcommand shows the built-in default rules.

Auto mode is enabled via `--permission-mode auto` or the `CLAUDE_CODE_ENABLE_AUTO_MODE`
env var. When active, a trained classifier automatically approves or denies tool
calls based on risk assessment rather than prompting the user.

### Since

v2.1.158

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master subcommand table |
| doc | [../param/046_permission_mode.md](../param/046_permission_mode.md) | Permission mode including `auto` |
| doc | [../param/081_enable_auto_mode.md](../param/081_enable_auto_mode.md) | Auto mode enable env var |

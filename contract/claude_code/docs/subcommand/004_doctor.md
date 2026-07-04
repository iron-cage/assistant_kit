# Subcommand: doctor

Check the health of your Claude Code auto-updater.

### Usage

```
claude doctor [options]
```

### Options

| Flag | Description |
|------|-------------|
| `-h`, `--help` | Display help |

### Sub-subcommands

None.

### Description

Diagnostic command that checks the health of the Claude Code auto-updater
mechanism. Reports issues with update infrastructure, permissions, and
connectivity that might prevent automatic updates from working.

### Since

v2.0.12 (2025-10-09)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master subcommand table |
| doc | [009_update.md](009_update.md) | Update/upgrade subcommand |
| doc | [../param/011_auto_updates.md](../param/011_auto_updates.md) | Auto-updates config key |
| doc | [../param/103_disable_doctor_command.md](../param/103_disable_doctor_command.md) | Env var to hide `/doctor` slash command |

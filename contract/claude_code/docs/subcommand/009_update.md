# Subcommand: update

Check for updates and install if available.

### Usage

```
claude update|upgrade [options]
```

### Options

| Flag | Description |
|------|-------------|
| `-h`, `--help` | Display help |

### Sub-subcommands

None.

### Description

Checks whether a newer version of Claude Code is available and installs it if
so. Respects the `preferredVersionSpec` setting if configured.

Alias: `claude upgrade` works identically to `claude update`.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master subcommand table |
| doc | [004_doctor.md](004_doctor.md) | Auto-updater health check |
| doc | [005_install.md](005_install.md) | Specific version installation |
| doc | [../params/011_auto_updates.md](../params/011_auto_updates.md) | `autoUpdates` config key |
| doc | [../params/050_preferred_version_spec.md](../params/050_preferred_version_spec.md) | Preferred version spec |

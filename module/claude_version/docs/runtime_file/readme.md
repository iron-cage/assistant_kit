# Runtime File Doc Entity

### Scope

- **Purpose**: Document all files created or managed by clv at known on-disk paths during normal operation.
- **Responsibility**: Track file paths, ownership, lifecycle, and durability of clv-managed runtime files.
- **In Scope**: Files written to disk by clv commands; their paths, owners, lifecycles, and crash durability.
- **Out of Scope**: `~/.claude/settings.json` and other files operated on by clv but owned by the external Claude Code system (→ `docs/feature/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `001_version_history_cache.md` | Version history API response cache file |
| `002_versions_directory.md` | Installed Claude Code binary versions directory |
| `003_binary_symlink.md` | Hot-swap symlink activating the current version |

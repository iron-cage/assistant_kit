# Runtime File: Versions Directory

### Scope

- **Purpose**: Document the on-disk directory storing installed Claude Code binary versions.
- **Responsibility**: Describe the directory path, per-version subdirectory format, owner functions, lifecycle triggers, and crash durability classification.
- **In Scope**: Directory path, per-version subdirectory layout, install/purge triggers, chmod-based lock state (555 locked / 755 unlocked).
- **Out of Scope**: Symlink retargeting (→ `runtime_file/003_binary_symlink.md`), version lock pattern layers (→ `pattern/001_version_lock.md`), chmod side effects (→ `pitfall/001_version_lock_chmod.md`).

### Abstract

Stores installed Claude Code binary versions, one subdirectory per version, used as the hot-swap source for `~/.local/bin/claude`. Directory permissions toggle between unlocked (755, writable by the auto-updater) and locked (555, read-only) to implement pinned-version protection.

### Path

`~/.local/share/claude/versions`

Resolution:
1. `$HOME/.local/share/claude/versions` — primary path, computed by `versions_dir_path()`
2. If `HOME` is unset, version installation and guard operations cannot proceed (see `feature/001_version_management.md`)

### Format

Directory containing one subdirectory per installed version (e.g., `2.1.78/`), each holding the extracted Claude Code binary and its supporting files. Not clv-specific — mirrors the layout the Claude Code auto-updater itself uses.

### Owner

`module/claude_version_core/src/version.rs`:
- `versions_dir_path()` (line 204) resolves the directory path.
- `perform_install()` (line 301) creates the target version's subdirectory during install.
- `purge_stale_versions()` (line 222) removes all subdirectories except the kept version (Layer 4 of the version lock pattern).
- `unlock_versions_dir()` (line 237) sets directory permissions to 755 (writable).
- `lock_version()` (line 254) sets directory permissions to 555 (read-only) after a pinned install.

### Lifecycle

- **Created:** On the first `.version.install` invocation, if the directory does not yet exist.
- **Subdirectory added:** Each successful `.version.install` adds (or overwrites) the target version's subdirectory.
- **Subdirectory removed:** `purge_stale_versions()` deletes all subdirectories other than the just-installed version, when a pinned install completes (Layer 4 defense against symlink retarget, see `pitfall/002_symlink_retarget.md`).
- **Permission toggled:** `unlock_versions_dir()` / `lock_version()` switch the directory between 755 and 555 around each install cycle.
- **Never fully deleted by clv:** individual version subdirectories are purged, but the parent directory itself is not removed.

### Durability

**Classification:** recoverable

A missing or emptied versions directory does not lose user data — it only removes cached installed binaries. The next `.version.install` re-downloads and recreates the needed subdirectory. Recovery requires network access; there is no local backup.

### Features

| File | Relationship |
|------|-------------|
| [feature/001_version_management.md](../feature/001_version_management.md) | `.version.install`, `.version.guard` create/purge/lock this directory |
| [feature/009_path_discovery.md](../feature/009_path_discovery.md) | `.paths` command that reports this path |

### Patterns

| File | Relationship |
|------|-------------|
| [pattern/001_version_lock.md](../pattern/001_version_lock.md) | Layers 3–4 use this directory's permissions and contents |

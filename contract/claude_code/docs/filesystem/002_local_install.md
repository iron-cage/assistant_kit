# Filesystem: Local Install

### Scope

- **Purpose**: Document the launcher binary and versioned binary paths in `~/.local/`.
- **Responsibility**: Authoritative instance for the `~/.local/` cluster — the launcher at `~/.local/bin/claude` and versioned binaries at `~/.local/share/claude/versions/`.
- **In Scope**: `~/.local/bin/claude` (launcher, on `$PATH`); `~/.local/share/claude/versions/` (versioned binaries subject to chmod lock); path resolution; version lock chmod operations.
- **Out of Scope**: `~/.claude/` settings and conversation storage (→ [001_claude_home.md](001_claude_home.md)); version lock settings keys (→ [`../settings/003_version_lock.md`](../settings/003_version_lock.md)).

### Paths

| Path | Type | Access | Used By | Purpose |
|------|------|--------|---------|---------|
| `~/.local/bin/claude` | file | R/del | `.version.install`, `.version.guard`, `.version.show` | Launcher binary; resolved via `which claude`, fallback `~/.local/bin/claude` |
| `~/.local/share/claude/versions/` | dir | chmod | `.version.install`, `.version.guard` | Versioned binaries; `chmod 555` (locked) or `755` (unlocked) |

### Resolution

| Path | Resolution Method |
|------|-------------------|
| `~/.local/bin/claude` | `which claude` (preferred); falls back to `$HOME/.local/bin/claude` |
| `~/.local/share/claude/versions/` | Hardcoded: `$HOME/.local/share/claude/versions` |

### Version Lock chmod Operations

The `versions/` directory is subject to chmod-based version locking (layer 3 of the version lock protocol):

| Operation | chmod | Effect |
|-----------|-------|--------|
| Lock version | `chmod 555` | Makes directory read-only; prevents installer from writing new binaries |
| Unlock version | `chmod 755` | Allows installer to write; must unlock before install, re-lock after |

Before any install, layer 3 is always unlocked (`chmod 755`) so the installer can write. After install, it is re-locked if a pinned version is configured.

See [`../settings/003_version_lock.md`](../settings/003_version_lock.md) for the full 6-layer version lock protocol.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Filesystem master index: full directory tree, path reference table |
| settings | [`../settings/003_version_lock.md`](../settings/003_version_lock.md) | Version lock: all 6 layers including this chmod layer |
| source | `../../../../module/claude_version_core/src/version.rs` | `hot_swap_binary()`, `versions_dir_path()` |
| doc | `../../../../module/claude_version/docs/feature/001_version_management.md` | Version management feature doc |

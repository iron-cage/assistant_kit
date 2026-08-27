# Filesystem: Local Install

### Scope

- **Purpose**: Document the launcher binary and versioned binary paths in `~/.local/`.
- **Responsibility**: Authoritative instance for the `~/.local/` cluster — the launcher at `~/.local/bin/claude` and versioned binaries at `~/.local/share/claude/versions/`.
- **In Scope**: `~/.local/bin/claude` (launcher, on `$PATH`); `~/.local/share/claude/versions/` (versioned binaries subject to chmod lock); path resolution; version lock chmod operations.
- **Out of Scope**: `~/.claude/` settings and conversation storage (→ [001_claude_home.md](001_claude_home.md)); version lock settings keys (→ [`../settings/003_version_lock.md`](../settings/003_version_lock.md)).

### Paths

| Path | Type | Access | Used By | Purpose |
|------|------|--------|---------|---------|
| `~/.local/bin/claude` | **symlink** | R/rename/del | `.version.install`, `.version.guard`, `.version.show` | Launcher; a symlink into `versions/`, not a copied binary |
| `~/.local/bin/claude.preinstall` | symlink | W/del | `.version.install` | Reversible rename-aside sidecar; exists only mid-install |
| `~/.local/share/claude/versions/` | dir | chmod | `.version.install`, `.version.guard` | Versioned binaries; `chmod 555` (locked) or `755` (unlocked) |
| `~/.local/share/claude/versions/{ver}` | file | R/W | installer | One executable per installed version, named by bare version |

### The Launcher Is a Symlink

`~/.local/bin/claude` is a **symbolic link** into the versions directory, not a copy of the
binary. An earlier revision typed it `file`, which obscures two consequences:

1. **The link target names the installed version.** `get_version_from_symlink()` reads it
   rather than executing anything — that is why version detection is instant and works even
   when the binary would refuse to run.
2. **Deleting the launcher does not free disk.** The bytes live in `versions/`.

```bash
ls -la ~/.local/bin/claude
# → ~/.local/bin/claude -> ~/.local/share/claude/versions/2.1.220
readlink ~/.local/bin/claude | xargs basename    # → 2.1.220, the installed version
```

Observed on a `installMethod: native` install. Whether every install method produces a
symlink here is ❓ Uncertain — only `native` was surveyed.

### The `.preinstall` Sidecar

Before an install, the launcher is **renamed aside** to `{path}.preinstall` rather than
deleted, so a failed install can put it back — `Fix(BUG-016)`, because the installer can
refuse to install *while still exiting 0*, leaving no launcher and nothing to restore.
It is removed once the outcome is confirmed, so seeing one on disk means an install
was interrupted:

```bash
ls -la ~/.local/bin/claude.preinstall   # normally: No such file or directory
```

Rename preserves the inode, so running sessions are unaffected either way (Unix open-file
semantics). Only if the rename itself fails does the code fall back to `remove_file`.

### Resolution

| Path | Resolution Method |
|------|-------------------|
| `~/.local/bin/claude` | `binary_symlink_path()` — hardcoded `$HOME/.local/bin/claude`. **Exception:** `hot_swap_binary()` alone prefers `which claude` and falls back to that constant, so a launcher elsewhere on `$PATH` is swapped correctly |
| `~/.local/share/claude/versions/` | `versions_dir_path()` — hardcoded `$HOME/.local/share/claude/versions` |
| `~/.claude/.transient/version_history_cache.json` | `version_history_cache_path()` — see [001_claude_home.md](001_claude_home.md) |

Neither constant honours `CLAUDE_CONFIG_DIR`; it relocates `~/.claude/` only, never
`~/.local/`. See [`../param/154_config_dir.md`](../param/154_config_dir.md).

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

# Runtime File: Binary Symlink

### Scope

- **Purpose**: Document the on-disk symlink that clv hot-swaps to activate a specific installed Claude Code version.
- **Responsibility**: Describe the symlink path, target resolution, owner functions, lifecycle triggers, and crash durability classification.
- **In Scope**: Symlink path, retarget mechanism, read/write owner functions, auto-updater interaction.
- **Out of Scope**: Versions directory content and lock state (→ `runtime_file/002_versions_directory.md`), symlink retarget bypass vector (→ `pitfall/002_symlink_retarget.md`).

### Abstract

A symlink at a fixed path that the `claude` command resolves through. clv retargets it to point at a specific version's binary inside the versions directory to hot-swap the active version without reinstalling.

### Path

`~/.local/bin/claude`

Resolution:
1. `$HOME/.local/bin/claude` — primary path
2. If `HOME` is unset, version commands relying on symlink state cannot proceed.

### Format

A symlink (not a regular file) pointing at a binary inside `~/.local/share/claude/versions/<version>/`. Its target path — not its own path — encodes the active version.

### Owner

`module/claude_version_core/src/version.rs`:
- `hot_swap_binary()` moves the symlink aside (to `~/.local/bin/claude.preinstall`) so the installer can lay down a fresh one; `restore_swapped_binary()` settles the sidecar afterward — restored on install failure, discarded on verified success (BUG-016).
- `get_version_from_symlink()` reads the current symlink target to determine the active version.

### Lifecycle

- **Created/retargeted:** by the official installer during `.version.install`; when Claude Code processes are running, `hot_swap_binary()` first moves the existing symlink aside so the installer's write is clean.
- **Read:** On `.status`, `.version.show`, and other commands that report the active version, via `get_version_from_symlink()`.
- **Externally retargeted:** The Claude Code auto-updater can also retarget this symlink outside of clv's control — see `pitfall/002_symlink_retarget.md` for the resulting bypass vector.
- **Moved aside during install, never left deleted (BUG-016):** during `.version.install`, clv renames the symlink to a `.preinstall` sidecar and either restores it (failed install) or deletes the sidecar (verified success) — a failed install no longer strands the host without a launcher.

### Durability

**Classification:** recoverable

A missing symlink breaks the `claude` command until repaired. Recovery is a single `.version.install` or hot-swap invocation, which recreates the symlink pointing at the appropriate version; no data is permanently lost.

### Features

| File | Relationship |
|------|-------------|
| [feature/001_version_management.md](../feature/001_version_management.md) | `.version.install`, `.version.guard` implement retarget/read |
| [feature/009_path_discovery.md](../feature/009_path_discovery.md) | `.version.paths` command that reports this path |

### Pitfalls

| File | Relationship |
|------|-------------|
| [pitfall/002_symlink_retarget.md](../pitfall/002_symlink_retarget.md) | Auto-updater can retarget this symlink to bypass the version lock |

# Pitfall: Auto-Updater Symlink Retarget

### Scope

- **Purpose**: Document the symlink retarget bypass vector in the version lock pattern.
- **Responsibility**: Describe the trap, observable failure, and mitigation for cached binary retargeting.
- **In Scope**: Symlink retarget mechanism, cached binaries in versions directory, Layer 4 purge defense.
- **Out of Scope**: chmod side effects (-> `pitfall/001_version_lock_chmod.md`), full lock pattern design (-> `pattern/001_version_lock.md`).

### Trap

After applying `chmod 555` to the versions directory (Layer 3), the incorrect assumption is that the pinned version is fully protected. The auto-updater cannot download new binaries into the protected directory, but it CAN retarget the `~/.local/bin/claude` symlink to a previously cached binary that already exists in the directory.

### Failure

The pinned version is silently replaced despite the permissions lock:
- `~/.local/bin/claude` symlink now points to a different (previously cached) binary
- `claude --version` reports a version different from the pinned preference
- No error is produced because no write to the protected directory occurred
- `.version.guard` detects the drift and can recover, but between guard runs the wrong version is active

### Mitigation

Layer 4 of the version lock pattern: after a successful pinned install, purge all other cached binaries from `~/.local/share/claude/versions/`. This eliminates retarget candidates entirely. Combined with Layer 3 (chmod 555), the auto-updater can neither download new binaries nor retarget to old ones.

Cost: purging is destructive. If a different version is needed later, it requires a full re-download.

### Patterns

| File | Relationship |
|------|-------------|
| [pattern/001_version_lock.md](../pattern/001_version_lock.md) | Layer 4 design that mitigates this pitfall |

### Features

| File | Relationship |
|------|-------------|
| [feature/001_version_management.md](../feature/001_version_management.md) | .version.install implements the purge |

### Sources

| File | Relationship |
|------|-------------|
| `../../src/commands/version.rs` | Binary purge logic in perform_install() |

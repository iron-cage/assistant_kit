# Pitfall: Version Lock chmod Side Effects

### Scope

- **Purpose**: Document the side effects of using chmod 555 as a version lock layer.
- **Responsibility**: Describe the trap, observable failure, and mitigation for permissions-based version lock.
- **In Scope**: chmod 555 on versions directory, manual operation lockout, unlock/restore procedure.
- **Out of Scope**: Other version lock layers (-> `pattern/001_version_lock.md`), auto-updater symlink bypass (-> `pitfall/002_symlink_retarget.md`).

### Trap

Layer 3 of the version lock pattern applies `chmod 555` to `~/.local/share/claude/versions/` to prevent the auto-updater from writing new binaries. The incorrect assumption is that this selectively blocks the auto-updater while leaving other operations unaffected.

### Failure

`chmod 555` blocks ALL write operations on the directory, not just the auto-updater:
- Manual `claude update` fails with permission denied
- Any tool or script attempting to write to the versions directory fails
- The user must remember to `chmod 755` before manual operations and restore `chmod 555` afterward
- Forgetting to restore permissions after manual work silently disables Layer 3 protection

### Mitigation

1. Document the unlock/operate/restore procedure in operational guides
2. Use `.version.install` (which handles chmod transitions automatically) instead of manual operations when possible
3. `.version.guard` also handles chmod restoration — running it periodically re-establishes the lock

### Patterns

| File | Relationship |
|------|-------------|
| [pattern/001_version_lock.md](../pattern/001_version_lock.md) | Layer 3 design where this pitfall originates |

### Sources

| File | Relationship |
|------|-------------|
| `../../src/commands/version.rs` | chmod 555/755 transitions in perform_install() |

### Tests

| File | Relationship |
|------|-------------|
| [`../../tests/docs/pitfall/001_version_lock_chmod.md`](../../tests/docs/pitfall/001_version_lock_chmod.md) | Regression test spec |

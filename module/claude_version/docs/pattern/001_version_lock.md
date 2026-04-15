# Pattern: Version Lock

### Scope

- **Purpose**: Document the 5-layer version lock strategy applied after installing a pinned Claude Code version.
- **Responsibility**: Describe the problem, solution layers, applicability, and consequences of the version lock design.
- **In Scope**: 5 lock layers for pinned versions, lock removal for `latest`, layer ordering rationale, symlink-retarget loophole.
- **Out of Scope**: Version install execution (→ `feature/001_version_management.md`), version guard recovery (→ `feature/001_version_management.md`).

### Problem

Claude Code's auto-updater can silently upgrade the installed version, breaking pinned-version workflows. The auto-updater has multiple bypass vectors:

1. `autoUpdates` setting can be ignored by the updater in some scenarios
2. Even with the `versions/` directory chmod-555 (no new writes), the auto-updater can retarget `~/.local/bin/claude` to a previously cached binary without writing to the protected directory
3. Environment variables need to be set in Claude's own settings to be visible to auto-update subprocesses

No single protection layer is sufficient — each layer closes a specific bypass vector.

### Solution

After a successful `.version.install` for a **pinned** version, apply all 5 layers in order:

1. **`autoUpdates = false`** in `settings.json` — soft lock via the Claude settings API (unreliable on its own, but disables the most common code path)
2. **`env.DISABLE_AUTOUPDATER = "1"`** in `settings.json` — official environment variable recognized by the Anthropic auto-updater; stored in Claude's own settings so it is visible to auto-update subprocesses
3. **`chmod 555 ~/.local/share/claude/versions/`** — hard lock on the versions directory; prevents new binary writes
4. **Purge all other cached binaries** from `~/.local/share/claude/versions/` — closes the symlink-retarget loophole (Layer 3 blocks new downloads but cannot prevent retargeting to an already-cached binary)
5. **Store `preferredVersionSpec` + `preferredVersionResolved`** in `settings.json` — recovery signal; `.version.guard` reads these to detect drift and reinstall if any of Layers 1–4 are bypassed or manually undone

For **`latest`**, the lock is reversed:
1. `autoUpdates = true`
2. Remove `DISABLE_AUTOUPDATER` from the `env` block
3. `chmod 755 ~/.local/share/claude/versions/` (unlock)

### Applicability

This pattern applies whenever:
- A specific Claude Code version must be pinned for reproducibility
- Multiple lock layers are needed because any single layer has a known bypass vector
- A recovery mechanism (`preferredVersionResolved`) is required to restore after accidental or automatic override

This pattern does not apply when tracking `latest` is desired — for `latest`, the pattern is inverted to remove all locks.

### Consequences

**Benefits:**
- Each layer closes a specific bypass vector; together they create defense in depth
- Layer 5 (recovery signal) enables automatic re-pinning via `.version.guard` even after all other layers are bypassed
- Purging cached binaries (Layer 4) prevents silent retargeting

**Costs:**
- `chmod 555` on the versions directory blocks not just the auto-updater but also manual operations — the user must `chmod 755` before manual work and restore afterwards
- Layer 4 (purging cached binaries) is destructive: it permanently removes cached binaries, requiring a full re-download if a different version is needed

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| feature | [feature/001_version_management.md](../feature/001_version_management.md) | .version.install and .version.guard that apply/read the lock |
| source | `../../src/commands.rs` | Lock application in perform_install() |
| source | `../../src/settings_io.rs` | settings.json write for layers 1, 2, 5 |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | FR-15 (5-layer version lock), FR-15a (nested JSON preservation), FR-17 (preferred version persistence) |

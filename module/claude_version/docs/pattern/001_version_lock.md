# Pattern: Version Lock

### Scope

- **Purpose**: Document the 8-layer version lock strategy applied after installing a pinned Claude Code version.
- **Responsibility**: Describe the problem, solution layers, applicability, and consequences of the version lock design.
- **In Scope**: 8 lock layers for pinned versions, lock removal for `latest`, layer ordering rationale, symlink-retarget loophole.
- **Out of Scope**: Version install execution (→ `feature/001_version_management.md`), version guard recovery (→ `feature/001_version_management.md`).

### Problem

Claude Code's auto-updater can silently upgrade the installed version, breaking pinned-version workflows. The auto-updater has multiple bypass vectors:

1. `autoUpdates` setting can be ignored by the updater in some scenarios
2. Even with the `versions/` directory chmod-555 (no new writes), the auto-updater can retarget `~/.local/bin/claude` to a previously cached binary without writing to the protected directory
3. Environment variables need to be set in Claude's own settings to be visible to auto-update subprocesses

No single protection layer is sufficient — each layer closes a specific bypass vector.

### Solution

After a successful `.version.install` for a **pinned** version, apply all 8 layers in order:

1. **`autoUpdates = false`** in `settings.json` — soft lock via the Claude settings API (unreliable on its own, but disables the most common code path)
2. **`env.DISABLE_AUTOUPDATER = "1"`** in `settings.json` — official environment variable recognized by the Anthropic auto-updater; stored in Claude's own settings so it is visible to auto-update subprocesses
3. **`chmod 555 ~/.local/share/claude/versions/`** — hard lock on the versions directory; prevents new binary writes
4. **Purge all other cached binaries** from `~/.local/share/claude/versions/` — closes the symlink-retarget loophole (Layer 3 blocks new downloads but cannot prevent retargeting to an already-cached binary)
5. **Store `preferredVersionSpec` + `preferredVersionResolved`** in `settings.json` — recovery signal; `.version.guard` re-resolves `preferredVersionSpec` through the current alias table at guard time to determine the target semver; `preferredVersionResolved` is advisory for alias specs and authoritative only for concrete semver specs
6. **`autoUpdatesChannel = "stable"`** in `settings.json` — official channel selector; stops the updater from drifting onto a pre-release/beta channel while pinned
7. **`minimumVersion = <resolved semver>`** in `settings.json` — official soft update floor; stops a floor-less downgrade back below the pinned version
8. **`env.DISABLE_UPDATES = "1"`** in `settings.json` — official update-suppression variable, independent of `DISABLE_AUTOUPDATER`; stops a manually-invoked `claude update`

For **`latest`**, the lock is reversed:
1. `autoUpdates = true`
2. Remove `DISABLE_AUTOUPDATER` from the `env` block
3. `chmod 755 ~/.local/share/claude/versions/` (unlock)
4. Remove `autoUpdatesChannel`
5. Remove `minimumVersion`
6. Remove `DISABLE_UPDATES` from the `env` block

### Applicability

This pattern applies whenever:
- A specific Claude Code version must be pinned for reproducibility
- Multiple lock layers are needed because any single layer has a known bypass vector
- A recovery mechanism is required to restore after accidental or automatic override — `preferredVersionSpec` (the alias or semver) is the recovery driver; `.version.guard` re-resolves it through the current alias table at runtime to determine the target semver

This pattern does not apply when tracking `latest` is desired — for `latest`, the pattern is inverted to remove all locks.

### Consequences

**Benefits:**
- Each layer closes a specific bypass vector; together they create defense in depth
- Layer 5 (recovery signal) enables automatic re-pinning via `.version.guard` even after all other layers are bypassed
- Purging cached binaries (Layer 4) prevents silent retargeting

**Costs:**
- `chmod 555` on the versions directory blocks not just the auto-updater but also manual operations — the user must `chmod 755` before manual work and restore afterwards
- Layer 4 (purging cached binaries) is destructive: it permanently removes cached binaries, requiring a full re-download if a different version is needed

### Mechanism Coverage

Cross-references this repo's 8-layer lock (see § Solution) against every mechanism enumerated in the official upstream pattern's Solution list ([../../../../contract/claude_code/docs/pattern/001_version_pinning.md](../../../../contract/claude_code/docs/pattern/001_version_pinning.md), items 1-8).

| # | Official mechanism | Key(s) | Used by this repo? | Evidence |
|---|---------------------|--------|---------------------|----------|
| 1 | Channel selection | `autoUpdatesChannel` | ✅ Used | This is this repo's own Layer 6 — set to `"stable"` by `lock_version()` on pinned installs |
| 2 | Soft update floor | `minimumVersion` | ✅ Used | This is this repo's own Layer 7 — set to the resolved semver by `lock_version()` on pinned installs |
| 3 | Hard organizational bounds | `requiredMinimumVersion` / `requiredMaximumVersion` | ❌ Not used | No occurrence anywhere in `module/` |
| 4 | Update suppression | `DISABLE_AUTOUPDATER` / `DISABLE_UPDATES` | ✅ Used | `DISABLE_AUTOUPDATER` is Layer 2 above; `DISABLE_UPDATES` is this repo's own Layer 8, set by `lock_version()` on pinned installs |
| 5 | Install-method restriction | `installMethod` | ❌ Not used | No occurrence anywhere in `module/` |
| 6 | Install-time version selection | `claude install` / bootstrap script / package managers / npm | ⚠️ One of four paths | Only the bootstrap curl script path is used, inside `perform_install()`; `claude install`, npm, and package-manager installs are never invoked by this repo's tooling |
| 7 | Recovery bridge (flagged non-official by the upstream doc itself) | `preferredVersionSpec` / `preferredVersionResolved` | ✅ Used | This IS this repo's own Layer 5 |
| 8 | Integrity complement | `manifest.json`, codesign/GPG/Authenticode | ❌ Not used | No occurrence anywhere in `module/` |

This repo also sets the plain `autoUpdates` boolean ([../../../../contract/claude_code/docs/param/011_auto_updates.md](../../../../contract/claude_code/docs/param/011_auto_updates.md)) as Layer 1 above — an official Claude Code parameter, but a separate one from the 8 items above (the upstream pattern's item 1 is the *channel* selector `autoUpdatesChannel`, not this boolean).

This repo's lock additionally applies enforcement with no official upstream equivalent: `chmod 555`/`755` on the versions directory (Layer 3) and purging cached binaries to close the symlink-retarget loophole (Layer 4) — see [pitfall/002_symlink_retarget.md](../pitfall/002_symlink_retarget.md).

### Features

| File | Relationship |
|------|-------------|
| [feature/001_version_management.md](../feature/001_version_management.md) | .version.install and .version.guard that apply/read the lock |

### Params

| File | Relationship |
|------|-------------|
| [../../../../contract/claude_code/docs/param/011_auto_updates.md](../../../../contract/claude_code/docs/param/011_auto_updates.md) | Official `autoUpdates` boolean this repo's Layer 1 sets directly |

### Patterns

| File | Relationship |
|------|-------------|
| [../../../../contract/claude_code/docs/pattern/001_version_pinning.md](../../../../contract/claude_code/docs/pattern/001_version_pinning.md) | Official upstream pinning mechanisms this repo's lock enforces on top of |

### Pitfalls

| File | Relationship |
|------|-------------|
| [pitfall/001_version_lock_chmod.md](../pitfall/001_version_lock_chmod.md) | chmod 555 side effects on manual operations |
| [pitfall/002_symlink_retarget.md](../pitfall/002_symlink_retarget.md) | Cached binary retarget bypass vector |

### Sources

| File | Relationship |
|------|-------------|
| `../../../claude_version_core/src/version.rs` | Lock application: `perform_install()` calls `lock_version()` |
| `../../../claude_version_core/src/settings_io.rs` | `settings.json` read/write primitives used by layers 1, 2, 5, 6, 7, 8 |

### Provenance

| Source | Notes |
|--------|-------|
| `spec.md` (deleted) | FR-15, FR-15a, FR-17 |

### Tests

| File | Relationship |
|------|-------------|
| [tests/docs/pattern/001_version_lock.md](../../tests/docs/pattern/001_version_lock.md) | Pattern test spec |

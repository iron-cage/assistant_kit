# Settings: Version Lock

### Scope

- **Purpose**: Document the version lock protocol — the 3-layer protection system and preferred version storage keys.
- **Responsibility**: Authoritative instance for version lock operations — all 3 layers, the `preferredVersionSpec`/`preferredVersionResolved` keys, and the install/guard workflow.
- **In Scope**: Layer 1 (`autoUpdates: false`), Layer 2 (`env.DISABLE_AUTOUPDATER`), Layer 3 (chmod 555 on versions dir); `preferredVersionSpec` and `preferredVersionResolved` keys; install/guard sequence.
- **Out of Scope**: `~/.local/share/claude/versions/` path (→ [`../filesystem/002_local_install.md`](../filesystem/002_local_install.md)); atomic write protocol (→ [001_global_settings.md](001_global_settings.md)).

### Version Lock Filesystem Operations

Pinned versions apply three protection layers:

| Layer | Path | Operation | Pinned | Latest |
|-------|------|-----------|--------|--------|
| 1 | `~/.claude/settings.json` | Set `autoUpdates` | `false` | `true` |
| 2 | `~/.claude/settings.json` | Set `env.DISABLE_AUTOUPDATER` | `"1"` | (removed) |
| 3 | `~/.local/share/claude/versions/` | `chmod` | `555` (locked) | `755` (unlocked) |

Before any install, Layer 3 is always unlocked (`chmod 755`) so the installer can write. After install, it is re-locked for pinned versions.

### Preferred Version Storage

Two keys written by `.version.install` on every successful exit (including idempotent skip):

| Key | Type | Example | Purpose |
|-----|------|---------|---------|
| `preferredVersionSpec` | string | `"stable"`, `"2.1.78"` | User's original request (alias or semver) |
| `preferredVersionResolved` | string \| null | `"2.1.78"`, `null` | Concrete semver at install time; `null` for `latest` |

`preferredVersionSpec` preserves the user's intent (they asked for `"stable"`, not a specific version). `preferredVersionResolved` is the concrete version that was actually installed, used by `.version.guard` to verify the current binary matches the pinned version.

### Install Sequence

`.version.install` follows this order:
1. Unlock Layer 3: `chmod 755` on versions directory
2. Run installer (downloads and installs binary)
3. Write `preferredVersionSpec` and `preferredVersionResolved` to settings
4. If pinned: apply Layer 1 (`autoUpdates: false`), Layer 2 (`DISABLE_AUTOUPDATER=1`), and re-lock Layer 3 (`chmod 555`)

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Settings master index: full parameter table |
| settings | [001_global_settings.md](001_global_settings.md) | `autoUpdates`, `env` keys and atomic write protocol |
| filesystem | [`../filesystem/002_local_install.md`](../filesystem/002_local_install.md) | `~/.local/share/claude/versions/` path and chmod operations |
| doc | `../../../../module/claude_version/docs/pattern/001_version_lock.md` | Version lock pattern feature doc |
| doc | [../pattern/001_version_pinning.md](../pattern/001_version_pinning.md) | Full official version-pinning landscape (channels, floors/ceilings, install methods) |
| source | `../../../../module/claude_version/src/commands.rs` | Version lock implementation |

# File Formats and Write Protocols

Internal structure and write semantics for files that claude_manager manages.

### Scope

- In scope: JSON structure, write protocols, and lock operations for managed files
- Out of scope: filesystem paths and directory layout (see [filesystem.md](filesystem.md))

### Atomic Write Protocol

Settings modifications use a temp-file rename pattern to prevent corruption:

1. Write new content to `~/.claude/settings.json.tmp`
2. Rename `settings.json.tmp` → `settings.json` (atomic on same filesystem)
3. On failure: `settings.json.tmp` is orphaned (no data loss to original)

All commands that modify settings (`.settings.set`, `.version.install`, `.version.guard`) use this protocol via the `set_setting()` function.

### Settings JSON Structure

`~/.claude/settings.json` is a flat `{ "k1": v1, ... }` object with nested object preservation:

```json
{
  "theme": "dark",
  "autoUpdates": false,
  "preferredVersionSpec": "stable",
  "preferredVersionResolved": "2.1.78",
  "env": {
    "DISABLE_AUTOUPDATER": "1"
  },
  "enabledPlugins": {}
}
```

**Key rules:**
- Top-level values: strings, numbers, booleans, null (hand-rolled parser, no serde)
- Nested objects (`env`, `enabledPlugins`): captured as raw JSON strings, output verbatim
- Only `env` sub-object is actively manipulated (set/remove individual env vars)
- Type inference on write: exact `"true"`/`"false"` → bool; `"null"` → raw null; values starting with `{`/`[` (after left-trim) → raw JSON; `i64`/`f64`-parseable → number; all others → string

### Version Lock Filesystem Operations

Pinned versions apply three protection layers, two of which involve filesystem writes:

| Layer | Path | Operation | Pinned | Latest |
|-------|------|-----------|--------|--------|
| 1 | `settings.json` | Set `autoUpdates` | `false` | `true` |
| 2 | `settings.json` | Set `env.DISABLE_AUTOUPDATER` | `"1"` | (removed) |
| 3 | `~/.local/share/claude/versions/` | `chmod` | `555` | `755` |

Before install, layer 3 is always unlocked (`chmod 755`) so the installer can write. After install, it is re-locked for pinned versions.

### Preferred Version Storage

Two keys written by `.version.install` on every successful exit (including idempotent skip):

| Key | Type | Example | Purpose |
|-----|------|---------|---------|
| `preferredVersionSpec` | string | `"stable"`, `"2.1.78"` | User's original request (alias or semver) |
| `preferredVersionResolved` | string or null | `"2.1.78"`, `null` | Concrete semver at install time; `null` for `latest` |

### Account Active Marker

`~/.claude/accounts/_active` is a single-line plain text file containing the active account name. Written by `.account.switch`, read by `.account.status` and `.status`.

### Cross-References

- [filesystem.md](filesystem.md) — path locations and directory tree
- [feature/003_settings_management.md](../../module/claude_manager/docs/feature/003_settings_management.md) — settings JSON, nested preservation
- [pattern/001_version_lock.md](../../module/claude_manager/docs/pattern/001_version_lock.md) — version lock
- [feature/001_version_management.md](../../module/claude_manager/docs/feature/001_version_management.md) — preference persistence
- [settings_io.rs](../src/settings_io.rs) — `set_setting()`, `get_setting()`, `read_all_settings()`, `infer_type()`

# Settings: Global Settings

### Scope

- **Purpose**: Document `~/.claude/settings.json` — user-global configuration, atomic write protocol, and type inference rules.
- **Responsibility**: Authoritative instance for global settings — JSON structure, all key-value semantics, write protocol, type inference on write.
- **In Scope**: `~/.claude/settings.json` structure; atomic temp-file rename protocol; type inference rules (`"true"` → bool, etc.); nested object preservation; all global-scope keys.
- **Out of Scope**: Project-level settings (→ [002_project_settings.md](002_project_settings.md)); version lock keys and chmod operations (→ [003_version_lock.md](003_version_lock.md)); filesystem path for the file (→ [`../filesystem/001_claude_home.md`](../filesystem/001_claude_home.md)).

### Structure

```json
{
  "theme": "dark",
  "autoUpdates": false,
  "preferredVersionSpec": "stable",
  "preferredVersionResolved": "2.1.78",
  "env": {
    "DISABLE_AUTOUPDATER": "1"
  },
  "enabledPlugins": {},
  "model": "sonnet",
  "effortLevel": "high",
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [{ "type": "command", "command": "/path/to/hook.sh" }]
      }
    ]
  },
  "skipDangerousModePermissionPrompt": false,
  "voiceEnabled": false,
  "fileCheckpointingEnabled": false,
  "remoteControlAtStartup": false
}
```

### Atomic Write Protocol

All modifications use a temp-file rename pattern:

1. Write new content to `~/.claude/settings.json.tmp`
2. Rename `settings.json.tmp` → `settings.json` (atomic on same filesystem)
3. On failure: `settings.json.tmp` orphaned (no data loss to original)

All commands that modify settings (`.settings.set`, `.version.install`, `.version.guard`) use this protocol via the `set_setting()` function.

### Type Inference on Write

When writing a value via `.settings.set`, the value string is type-inferred:

| Input string | Written as |
|-------------|-----------|
| `"true"` or `"false"` | boolean |
| `"null"` | raw null |
| Starts with `{` or `[` (after left-trim) | raw JSON |
| Parseable as `i64` or `f64` | number |
| All other strings | string |

### Nested Object Preservation

Top-level values: strings, numbers, booleans, null (hand-rolled parser, no serde).
Nested objects (`env`, `enabledPlugins`, `hooks`, `mcpServers`, `permissions`): captured as raw JSON strings, output verbatim. Only the `env` sub-object is actively manipulated (individual key set/remove).

### Global-Only Keys

Keys valid only in `~/.claude/settings.json` (not in project settings):

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `theme` | string | `"dark"` | UI color theme |
| `autoUpdates` | bool | `true` | Auto-update binary on startup |
| `preferredVersionSpec` | string/null | `null` | Preferred version alias or semver |
| `preferredVersionResolved` | string/null | `null` | Concrete semver at last install |
| `env` | object | `{}` | Persistent env var overrides injected at startup |
| `enabledPlugins` | object | `{}` | Active plugin registry |
| `skipDangerousModePermissionPrompt` | bool | `false` | Suppress interactive dangerous mode confirmation |
| `voiceEnabled` | bool | `false` | Enable voice input/output |
| `fileCheckpointingEnabled` | bool | `false` | Save checkpoint copy of each file before editing |
| `remoteControlAtStartup` | bool | `false` | Open remote-control channel on startup |

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Settings master index: full parameter table, atomic write protocol |
| settings | [002_project_settings.md](002_project_settings.md) | Project-level settings (G+P keys and P-only keys) |
| settings | [003_version_lock.md](003_version_lock.md) | Version lock: `autoUpdates`, `env.DISABLE_AUTOUPDATER`, chmod layer |
| filesystem | [`../filesystem/001_claude_home.md`](../filesystem/001_claude_home.md) | `~/.claude/settings.json` and `settings.json.tmp` paths |
| source | `../../../../module/claude_version_core/src/settings_io.rs` | `set_setting()`, `get_setting()`, `read_all_settings()`, `infer_type()` |

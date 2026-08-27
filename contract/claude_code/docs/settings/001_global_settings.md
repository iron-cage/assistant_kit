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

Illustrative, not a capture of a real file. Note that `preferredVersionSpec` and
`preferredVersionResolved` appear here because they genuinely occur in `settings.json` —
but they are written by `clv`, not by `claude`. See § Global-Only Keys for which is which.

### Atomic Write Protocol

All modifications use a temp-file rename pattern:

1. Write new content to `~/.claude/settings.json.tmp`
2. Rename `settings.json.tmp` → `settings.json` (atomic on same filesystem)
3. On failure: `settings.json.tmp` orphaned (no data loss to original)

All commands that modify settings (`.settings.set`, `.version.install`, `.version.guard`) use this protocol via the `set_setting()` function.

### Type Inference on Write

When writing a value via `.settings.set`, the value string is type-inferred:

Checked in the order the source applies them (`infer_type()`), which matters for the
first row — a value starting `{` is never tested against `"true"`:

| # | Input string | Written as |
|---|-------------|-----------|
| 1 | Starts with `{` or `[` (after left-trim) | raw JSON |
| 2 | `"true"` or `"false"` | boolean |
| 3 | `"null"` | raw null |
| 4 | Parses as `i64`, **or** as `f64` *and* `is_finite()` | number |
| 5 | Everything else | string |

**Row 4's `is_finite()` guard is load-bearing, not a formality.** Rust's `f64::from_str`
accepts `inf`, `-inf`, `infinity`, and `NaN` — none of which is a legal JSON number. Without
the guard these were emitted bare and produced a settings file no JSON parser would read.
The guard sends them down row 5 instead, so they are stored as **strings**:

```bash
clv .settings.set someKey inf     # stored as "inf" (string), NOT bare inf
clv .settings.set someKey 1e400   # overflows to f64::INFINITY → also a string
clv .settings.set someKey 1.5     # stored as 1.5 (number)
```

Recorded in the source as `Fix(issue-infer-nan)`. An earlier revision of this table listed
row 4 as plain "Parseable as `i64` or `f64`", which predicts a corrupt file for `inf`.

### Nested Object Preservation

Top-level values: strings, numbers, booleans, null (hand-rolled parser, no serde).

**Any** nested object or array is captured as a raw JSON string and re-emitted verbatim.
An earlier revision of this section named five keys — `env`, `enabledPlugins`, `hooks`,
`mcpServers`, `permissions` — which reads as a whitelist. There is no key whitelist in the
implementation: `json_parse_flat_object` classifies by *value shape*, so a nested key that
did not exist when this document was written (`statusLine`, `permissions.additionalDirectories`,
anything a future release adds) is preserved on the same terms. That is the stronger and
the true guarantee; the five names were only the examples that happened to be present.

Only the `env` sub-object is actively manipulated (individual key set/remove).

Verify preservation of a key nobody enumerated:

```bash
clv .settings.get someUnrelatedKey   # read back after any .settings.set on another key
# nested objects survive verbatim regardless of name
```

`get_setting()` returns the stringified value whatever the underlying JSON type;
`get_string_setting()` returns `None` unless the stored value is genuinely a JSON string.
Use the latter when a numeric-looking string like `"2.1.220"` must not be confused with a
number.

### Global-Only Keys

Keys valid only in `~/.claude/settings.json` (not in project settings). The **Src** column
records who the key belongs to, per the quoted-literal binary scan in
[readme.md](readme.md) § Provenance — `CC` = confirmed in the `claude` binary,
`clv` = written by this repo's tooling and absent from the binary, `❓` = absent from both:

| Key | Src | Type | Default | Description |
|-----|-----|------|---------|-------------|
| `theme` | CC (19) | string | `"dark"` | UI color theme |
| `autoUpdates` | CC (2) | bool | `true` | Auto-update binary on startup |
| `env` | CC (59) | object | `{}` | Persistent env var overrides injected at startup |
| `enabledPlugins` | CC (5) | object | `{}` | Active plugin registry |
| `fileCheckpointingEnabled` | CC (4) | bool | `false` | Save checkpoint copy of each file before editing |
| `remoteControlAtStartup` | CC (4) | bool | `false` | Open remote-control channel on startup |
| `tui` | CC (1) | string | ❓ | ❓ Observed as `"fullscreen"`; semantics not characterized |
| `preferredVersionSpec` | **clv** | string/null | `null` | Preferred version alias or semver — read by `clv`, not by `claude` |
| `preferredVersionResolved` | **clv** | string/null | `null` | Concrete semver at last install — likewise |
| `skipDangerousModePermissionPrompt` | ❓ | bool | `false` | ❓ Unconfirmed; `permissionMode: "bypassPermissions"` is the confirmable mechanism |
| `voiceEnabled` | ❓ | bool | `false` | ❓ Unconfirmed |

The two `clv` rows sit in Claude Code's settings file but are not Claude Code's settings —
`settings.json` is a shared namespace, the same way `~/.claude/` itself is (see
[`../storage/003_root_files.md`](../storage/003_root_files.md) § Not Claude Code's).
Deleting them changes nothing about how `claude` starts; it un-pins `clv`.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Settings master index: full parameter table, atomic write protocol |
| settings | [002_project_settings.md](002_project_settings.md) | Project-level settings (G+P keys and P-only keys) |
| settings | [003_version_lock.md](003_version_lock.md) | Version lock: `autoUpdates`, `env.DISABLE_AUTOUPDATER`, chmod layer |
| filesystem | [`../filesystem/001_claude_home.md`](../filesystem/001_claude_home.md) | `~/.claude/settings.json` and `settings.json.tmp` paths |
| source | `../../../../module/claude_core/src/settings_io.rs` | `set_setting()`, `get_setting()`, `get_string_setting()`, `read_all_settings()`, `infer_type()` — the implementation moved here from `claude_version_core`, which earlier revisions still cited |

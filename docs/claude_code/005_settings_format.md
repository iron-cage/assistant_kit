# Claude Code: Settings Format

### Scope

- **Purpose**: Document the internal structure and write semantics for settings and credential files managed by claude_version.
- **Responsibility**: Authoritative reference for settings.json structure, atomic write protocol, version lock operations, and account active marker.
- **In Scope**: JSON structure, write protocols, lock operations, type inference rules, preferred version storage.
- **Out of Scope**: Filesystem paths and directory layout (→ [003_filesystem_layout.md](003_filesystem_layout.md)).

### Atomic Write Protocol

Settings modifications use a temp-file rename pattern to prevent corruption:

1. Write new content to `~/.claude/settings.json.tmp`
2. Rename `settings.json.tmp` → `settings.json` (atomic on same filesystem)
3. On failure: `settings.json.tmp` is orphaned (no data loss to original)

All commands that modify settings (`.settings.set`, `.version.install`, `.version.guard`) use this protocol via the `set_setting()` function.

### Settings JSON Structure

`~/.claude/settings.json` is a flat `{ "k1": v1, ... }` object with nested object preservation:

User-global `~/.claude/settings.json`:

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

Project-level `.claude/settings.local.json` (auto-managed by Claude Code):

```json
{
  "permissions": {
    "allow": [
      "Bash(npm run test:*)",
      "Bash(npx eslint:*)"
    ],
    "deny": [],
    "ask": []
  },
  "outputStyle": "default"
}
```

**Key rules:**
- Top-level values: strings, numbers, booleans, null (hand-rolled parser, no serde)
- Nested objects (`env`, `enabledPlugins`, `hooks`, `mcpServers`, `permissions`): captured as raw JSON strings, output verbatim
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

### Settings Config Parameter Table

Config keys read by `claude` at startup from `settings.json`. No CLI flag or env var form except where noted (config key overrides are lower precedence than CLI flags). User-global keys (`~/.claude/settings.json`) marked **G**; project-level keys (`.claude/settings.json` / `.claude/settings.local.json`) marked **P**; keys valid at both levels marked **G+P**.

| Key | Scope | Type | Default | Description |
|-----|-------|------|---------|-------------|
| `theme` | G | string | `"dark"` | UI color theme |
| `autoUpdates` | G | bool | `true` | Auto-update binary on startup |
| `preferredVersionSpec` | G | string/null | `null` | Preferred version alias or semver (e.g. `"stable"`, `"2.1.78"`) |
| `preferredVersionResolved` | G | string/null | `null` | Concrete semver resolved at last install; `null` for `latest` |
| `env` | G | object | `{}` | Persistent env var overrides injected at startup |
| `enabledPlugins` | G | object | `{}` | Active plugin registry |
| `model` | G+P | string | binary default | Persistent model preference; overridden by `--model` CLI flag |
| `effortLevel` | G+P | enum | `"medium"` | Persistent effort level (`low`/`medium`/`high`/`max`); overridden by `--effort` |
| `hooks` | G+P | object | `{}` | Hooks for `PreToolUse` / `PostToolUse` / `UserPromptSubmit` events |
| `mcpServers` | G+P | object | `{}` | Inline MCP server definitions (alternative to `--mcp-config` flag) |
| `permissionMode` | G+P | enum | `"default"` | Permission mode: `default` `acceptEdits` `bypassPermissions` `dontAsk` `plan` `auto` |
| `allowedTools` | G+P | string[] | all | Persistent allowlist of permitted tools (overrides default; also set by `--allowed-tools`) |
| `disallowedTools` | G+P | string[] | none | Persistent denylist of forbidden tools (also set by `--disallowed-tools`) |
| `skipDangerousModePermissionPrompt` | G | bool | `false` | Suppress interactive confirmation when dangerous mode is active |
| `voiceEnabled` | G | bool | `false` | Enable voice input and audio output |
| `permissions` | P | object | `{}` | Per-project tool allow/deny/ask rules; auto-managed by Claude Code during sessions |
| `outputStyle` | G+P | string | `"default"` | Terminal output visual rendering style |
| `fileCheckpointingEnabled` | G | bool | `false` | Save a checkpoint copy of each file before editing |
| `remoteControlAtStartup` | G | bool | `false` | Open remote-control channel on startup for IDE/orchestrator connections |

See [`params/readme.md`](params/readme.md) for the complete parameter table including CLI flags and env vars. Precedence: CLI arg > env var > settings config.

### Account Active Marker

`{credential_store}/_active` is a single-line plain text file containing the active account name, where `{credential_store}` resolves to `$PRO/.persistent/claude/credential/` when `$PRO` is a directory, or `$HOME/.persistent/claude/credential/` otherwise. Written by `.account.switch`, read by `.account.status` and `.status`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [`003_filesystem_layout.md`](003_filesystem_layout.md) | Path locations and directory tree |
| doc | [`params/readme.md`](params/readme.md) | Full parameter table including config keys, CLI flags, and env vars |
| doc | [`../../module/claude_version/docs/feature/003_settings_management.md`](../../module/claude_version/docs/feature/003_settings_management.md) | Settings JSON, nested preservation feature doc |
| doc | [`../../module/claude_version/docs/pattern/001_version_lock.md`](../../module/claude_version/docs/pattern/001_version_lock.md) | Version lock pattern |
| doc | [`../../module/claude_version/docs/feature/001_version_management.md`](../../module/claude_version/docs/feature/001_version_management.md) | Preference persistence feature doc |
| source | [`../../module/claude_version/src/settings_io.rs`](../../module/claude_version/src/settings_io.rs) | `set_setting()`, `get_setting()`, `read_all_settings()`, `infer_type()` |

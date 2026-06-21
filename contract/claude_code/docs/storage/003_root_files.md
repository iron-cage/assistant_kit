# Storage: Root Files

### Scope

- **Purpose**: Document the global files at the `~/.claude/` root that are not inside any subdirectory.
- **Responsibility**: Authoritative instance for root-level files — `history.jsonl`, `.credentials.json`, and `settings.json` — purpose, format, access patterns, and security considerations.
- **In Scope**: `history.jsonl` (global project index), `.credentials.json` (API tokens), `settings.json` (user settings), `cld-timeout-config.json` (timeout config), `stats-cache.json` (usage stats cache).
- **Out of Scope**: `projects/` directory (→ [001_projects_directory.md](001_projects_directory.md)); support directories (→ [002_support_directories.md](002_support_directories.md)); settings file format internals (→ [`../settings/`](../settings/readme.md)); credentials file format (→ [`../formats/002_credentials.md`](../formats/002_credentials.md)).

### Structure

```
~/.claude/
├── history.jsonl              # 1.1MB - Global project access index
├── .credentials.json          # ~1KB  - Active API authentication tokens
├── settings.json              # ~5KB  - User settings and configuration
├── cld-timeout-config.json    # <1KB  - Bash tool timeout configuration
└── stats-cache.json           # <1KB  - Usage statistics cache
```

### Contents

#### history.jsonl — Global Project Index (1.1MB)

**Purpose**: Track all project accesses and context across all sessions.
**Format**: Line-delimited JSON — one entry per conversation start.

```json
{
  "display": "https://www.youtube-transcript.io/api\nread page...",
  "pastedContents": {},
  "timestamp": 1758992388766,
  "project": "/home/alice/projects/consumer-app/module/reasoner"
}
```

**Growth**: Appends one entry per conversation start (~4,324 entries observed, ~254 bytes/entry, ~1.1MB total).
**Access frequency**: Medium — read at project start.
**Maintenance**: Can be truncated if very large; loses project history but not conversations.

See [`../formats/001_history_jsonl.md`](../formats/001_history_jsonl.md) for full field spec.

#### .credentials.json — Active API Tokens (~1KB)

**Purpose**: Store active API authentication tokens for Claude Code.
**Format**: Single JSON object with `claudeAiOauth` key.
**Access frequency**: High — read and written on token refresh.
**Security**: High sensitivity. Recommended permissions: `chmod 600 ~/.claude/.credentials.json`

```json
{ "claudeAiOauth": { "... authentication data ..." } }
```

Never delete unless intentionally deauthenticating. Written atomically by `.account.switch`. See [`../formats/002_credentials.md`](../formats/002_credentials.md) for format spec.

#### settings.json — User Settings (~5KB)

**Purpose**: User configuration for Claude Code behavior, model preferences, hooks, and env vars.
**Format**: Flat JSON object with nested object preservation.
**Access frequency**: High — read on every startup; written on settings changes and version install.
**Write protocol**: Atomic via temp file `settings.json.tmp` → rename.

Key groups:
- **Display**: `theme`, `outputStyle`
- **Updates**: `autoUpdates`, `preferredVersionSpec`, `preferredVersionResolved`
- **Behavior**: `model`, `effortLevel`, `permissionMode`, `allowedTools`, `disallowedTools`
- **Runtime**: `env`, `hooks`, `mcpServers`, `enabledPlugins`
- **Features**: `voiceEnabled`, `fileCheckpointingEnabled`, `remoteControlAtStartup`

See [`../settings/001_global_settings.md`](../settings/001_global_settings.md) for full key table and write protocol.

#### cld-timeout-config.json — Bash Tool Timeout Configuration (<1KB)

**Purpose**: Stores user-specific overrides for bash tool timeout behavior.
**Format**: JSON object with timeout-related keys.
**Access frequency**: Low — read on session startup.
**Maintenance**: Safe to delete; reverts to default timeout behavior.

#### stats-cache.json — Usage Statistics Cache (<1KB)

**Purpose**: Caches usage statistics and token counts for display in the status bar.
**Format**: JSON object with aggregated usage metrics.
**Access frequency**: Medium — updated during sessions.
**Maintenance**: Safe to delete; will be regenerated from session data.

### Security Summary

| File | Sensitivity | Recommended Permissions |
|------|-------------|------------------------|
| `.credentials.json` | High (API tokens) | `chmod 600` |
| `settings.json` | Medium (config + env vars) | `chmod 644` |
| `history.jsonl` | Medium (project paths) | `chmod 644` |

**Maintenance**: Never delete `.credentials.json` or `settings.json` during normal operation. `history.jsonl` can be truncated safely.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Storage master index: full directory structure |
| settings | [`../settings/001_global_settings.md`](../settings/001_global_settings.md) | settings.json structure, write protocol, key table |
| formats | [`../formats/001_history_jsonl.md`](../formats/001_history_jsonl.md) | history.jsonl entry schema |
| formats | [`../formats/002_credentials.md`](../formats/002_credentials.md) | .credentials.json structure |
| filesystem | [`../filesystem/001_claude_home.md`](../filesystem/001_claude_home.md) | Path resolution for all `~/.claude/` files |

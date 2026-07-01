# Settings

### Scope

- **Purpose**: Document the structure and write semantics for settings files managed by claude_version.
- **Responsibility**: Master file for the `settings` collection — lists all 3 settings file instances covering global user settings, project-level settings, and the version lock protocol.
- **In Scope**: settings.json structure, atomic write protocol, project-level settings, version lock operations, preferred version storage, type inference rules, settings config parameter table.
- **Out of Scope**: Filesystem paths and directory layout (→ [`../filesystem/`](../filesystem/readme.md)); credential file format (→ [`../format/002_credentials.md`](../format/002_credentials.md)); account active marker (→ [`../filesystem/003_credential_store.md`](../filesystem/003_credential_store.md)).

### Overview Table

| ID | Name | Responsibility |
|----|------|----------------|
| [001](001_global_settings.md) | Global Settings | `~/.claude/settings.json` — user-global config keys, atomic write protocol, type inference |
| [002](002_project_settings.md) | Project Settings | `.claude/settings.json` and `.claude/settings.local.json` — project-level permissions, model, hooks |
| [003](003_version_lock.md) | Version Lock | Version lock filesystem operations, preferredVersionSpec/preferredVersionResolved storage, chmod protection layers |

### Atomic Write Protocol

All settings modifications use a temp-file rename pattern to prevent corruption:

1. Write new content to `~/.claude/settings.json.tmp`
2. Rename `settings.json.tmp` → `settings.json` (atomic on same filesystem)
3. On failure: `settings.json.tmp` is orphaned (no data loss to original)

### Settings Config Parameter Table

Config keys read by `claude` at startup from `settings.json`. Scope: **G** = user-global only, **P** = project-level only, **G+P** = both. Precedence: CLI arg > env var > settings config.

| Key | Scope | Type | Default | Description |
|-----|-------|------|---------|-------------|
| `theme` | G | string | `"dark"` | UI color theme |
| `autoUpdates` | G | bool | `true` | Auto-update binary on startup |
| `preferredVersionSpec` | G | string/null | `null` | Preferred version alias or semver |
| `preferredVersionResolved` | G | string/null | `null` | Concrete semver resolved at last install |
| `env` | G | object | `{}` | Persistent env var overrides injected at startup |
| `enabledPlugins` | G | object | `{}` | Active plugin registry |
| `model` | G+P | string | binary default | Persistent model preference; overridden by `--model` |
| `effortLevel` | G+P | enum | `"medium"` | Persistent effort level (`low`/`medium`/`high`/`max`) |
| `hooks` | G+P | object | `{}` | Hooks for `PreToolUse` / `PostToolUse` / `UserPromptSubmit` events |
| `mcpServers` | G+P | object | `{}` | Inline MCP server definitions |
| `permissionMode` | G+P | enum | `"default"` | Permission mode: `default` `acceptEdits` `bypassPermissions` `dontAsk` `plan` `auto` |
| `allowedTools` | G+P | string[] | all | Persistent allowlist of permitted tools |
| `disallowedTools` | G+P | string[] | none | Persistent denylist of forbidden tools |
| `skipDangerousModePermissionPrompt` | G | bool | `false` | Suppress interactive confirmation in dangerous mode |
| `voiceEnabled` | G | bool | `false` | Enable voice input and audio output |
| `permissions` | P | object | `{}` | Per-project tool allow/deny/ask rules; auto-managed |
| `outputStyle` | G+P | string | `"default"` | Terminal output visual rendering style |
| `fileCheckpointingEnabled` | G | bool | `false` | Save checkpoint copy of each file before editing |
| `remoteControlAtStartup` | G | bool | `false` | Open remote-control channel on startup |

See [`../param/readme.md`](../param/readme.md) for the complete parameter table including CLI flags and env vars.

### Type-Specific Requirements

All `settings` doc instances must include:

1. **Title**: `# Settings: {File or Protocol Name}` — using `Settings` as the type prefix
2. **Scope** (H3): 4 required bullets — Purpose, Responsibility, In Scope, Out of Scope
3. **Structure** (H3): JSON example and field table for this settings context
4. **Key Rules** (H3): Type inference, write protocol, or lock operations specific to this instance
5. **Cross-References** (H3): Flat table with `Type | File | Responsibility` columns

### Cross-Collection Dependencies

**This entity depends on**:
- `../filesystem/` — path locations for settings.json and settings.json.tmp
- `../param/` — full parameter table (CLI flags and env vars that complement config keys)

**This entity consumed by**:
- `../../../../module/claude_version/docs/` — settings management and version lock feature docs
- `../../../../module/claude_version/src/settings_io.rs` — `set_setting()`, `get_setting()`, `read_all_settings()`, `infer_type()`

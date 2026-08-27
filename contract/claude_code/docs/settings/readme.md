# Settings Doc Entity

### Scope

- **Purpose**: Document the structure and write semantics for settings files managed by claude_version.
- **Responsibility**: Master file for the `settings` collection — lists all 3 settings file instances covering global user settings, project-level settings, and the version lock protocol.
- **In Scope**: settings.json structure, atomic write protocol, project-level settings, managed-settings.json enforcement keys, version lock operations, preferred version storage, type inference rules, settings config parameter table, and the provenance separating binary-confirmed keys from clv-managed ones.
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

Config keys stored in `settings.json`. Scope: **G** = user-global only, **P** = project-level only, **G+P** = both, **M** = managed settings only. Precedence: CLI arg > env var > settings config.

> ⚠️ **Not every key here is read by `claude`.** An earlier revision introduced this table as
> "config keys read by `claude` at startup", which is false for five of them — see
> § Provenance below. `settings.json` is a **shared file**, not a Claude-Code-only namespace:
> this repo's own `clv` writes version-management keys into it, and those keys do not exist
> in the binary at all.

| Key | Scope | Type | Default | Description |
|-----|-------|------|---------|-------------|
| `theme` | G | string | `"dark"` | UI color theme |
| `autoUpdates` | G | bool | `true` | Auto-update binary on startup |
| `autoUpdatesChannel` | G+P | string | `"latest"` | Release channel for auto-updates: `latest` or `stable` |
| `minimumVersion` | G+P | string (semver) | — | Update floor; blocks auto-update/`claude update` below this version |
| `requiredMinimumVersion` | M | string (semver) | — | Startup floor; Claude Code exits at launch if older |
| `requiredMaximumVersion` | M | string (semver) | — | Startup ceiling; Claude Code exits at launch if newer |
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
| `tui` | G | string | ❓ | ❓ Binary-confirmed; observed as `"fullscreen"`; semantics not characterized |
| `statusLine` | ❓ | ❓ | ❓ | ❓ Binary-confirmed (4 hits); not characterized |
| `fileCheckpointingEnabled` | G | bool | `false` | Save checkpoint copy of each file before editing |
| `remoteControlAtStartup` | G | bool | `false` | Open remote-control channel on startup |

See [`../param/readme.md`](../param/readme.md) for the complete parameter table including CLI flags and env vars.

### Provenance

Every key above was scanned as a quoted literal against the installed v2.1.220 binary, the
same method used for directory names in
[`../storage/002_support_directories.md`](../storage/002_support_directories.md) § Provenance.
A fabricated control returns 0; genuine keys return 2–72.

**Binary-confirmed** — the key exists as a literal in the binary:

| Key | Hits | | Key | Hits |
|-----|------|-|-----|------|
| `model` | 72 | | `enabledPlugins` | 5 |
| `env` | 59 | | `outputStyle` | 5 |
| `hooks` | 58 | | `fileCheckpointingEnabled` | 4 |
| `theme` | 19 | | `remoteControlAtStartup` | 4 |
| `mcpServers` | 16 | | `autoUpdates` | 2 |
| `permissions` | 8 | | `effortLevel` | 2 |
| `autoUpdatesChannel` | 5 | | | |

**clv-managed, not Claude Code's** — 0 hits in the binary, but written and read by this
repo's own tooling, which fully accounts for their presence in `settings.json`:

| Key | Binary | Written by |
|-----|--------|------------|
| `preferredVersionSpec` | 0 | `claude_version_core/src/version.rs` (`set_pinned_version`) |
| `preferredVersionResolved` | 0 | same |
| `minimumVersion` | 0 | `claude_version_core` |

Listing these as keys `claude` reads was the specific error the callout above corrects.

**Unconfirmed** — 0 hits, and nothing in this repo writes them either; only a
`params_catalog.rs` entry mapping a CLI flag to the key name:

| Key | Binary | Note |
|-----|--------|------|
| `skipDangerousModePermissionPrompt` | 0 | `DangerousMode` occurs 13×, so the *feature* exists; this exact key does not appear |
| `voiceEnabled` | 0 | `voice` occurs 575×, same situation |

❓ Uncertain, **not** refuted — a quoted-literal scan cannot distinguish "absent" from
"assembled at runtime" or "renamed by the minifier". Treat both as unverified until
someone sets one and observes an effect. For dangerous-mode suppression the confirmable
mechanism is `permissionMode: "bypassPermissions"` (213 hits), not this key.

**Binary-confirmed but undocumented here** — real keys this collection never listed:

| Key | Hits | Status |
|-----|------|--------|
| `tui` | 1 | Present in the surveyed `settings.json` as `"fullscreen"`; semantics not characterized |
| `statusLine` | 4 | Not characterized |

Re-run any row yourself:

```bash
V=~/.local/share/claude/versions/$(claude --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')
grep -ac '"model"' "$V"                 # → 72  (positive control)
grep -ac '"preferredVersionSpec"' "$V"  # → 0   (clv-managed)
grep -ac '"NEVER_REAL_KEY_XYZ"' "$V"    # → 0   (negative control)
```

Quote the key. A bare `grep -ac env` matches `environment` and every other substring — the
same trap documented in [`../storage/002_support_directories.md`](../storage/002_support_directories.md) § Provenance.

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
- `../../../../module/claude_core/src/settings_io.rs` — `set_setting()`, `get_setting()`, `get_string_setting()`, `read_all_settings()`, `infer_type()`

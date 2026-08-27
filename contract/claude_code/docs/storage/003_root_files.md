# Storage: Root Files

### Scope

- **Purpose**: Document the global files at the `~/.claude/` root that are not inside any subdirectory.
- **Responsibility**: Authoritative instance for root-level files — purpose, format, access patterns, security considerations, and the provenance evidence separating files the binary creates from files that merely share the directory.
- **In Scope**: `history.jsonl` (global project index), `.credentials.json` (API tokens), `settings.json` (user settings), `stats-cache.json` (usage stats cache), `CLAUDE.md` (global instructions), `.last-cleanup`, `.last-update-result.json`, `scheduled_tasks.json`, `launch.json`, `daemon.json`, `policy-limits.json`.
- **Out of Scope**: `projects/` directory (→ [001_projects_directory.md](001_projects_directory.md)); support directories (→ [002_support_directories.md](002_support_directories.md)); settings file format internals (→ [`../settings/`](../settings/readme.md)); credentials file format (→ [`../format/002_credentials.md`](../format/002_credentials.md)); files present in `~/.claude/` that the binary does not create (see § Not Claude Code's).

### Structure

```
~/.claude/
├── history.jsonl              # Global project access index
├── .credentials.json          # Active API authentication tokens
├── settings.json              # User settings and configuration
├── stats-cache.json           # Usage statistics cache
├── CLAUDE.md                  # Global instructions, injected into every session
├── .last-cleanup              # Bare ISO-8601 timestamp of the last cleanup sweep
├── .last-update-result.json   # Outcome of the last self-update attempt
├── scheduled_tasks.json       # Scheduled task definitions
├── launch.json                # Launch configuration
├── daemon.json                # Daemon configuration
└── policy-limits.json         # Policy limit configuration
```

Sizes are omitted deliberately — they are machine-specific and drift continuously. An
earlier revision stated `history.jsonl # 1.1MB` and "~4,324 entries observed"; on the
machine used for this revision it is 4.26MB across 14133 entries. Check your own with
`ls -la ~/.claude/*.json ~/.claude/history.jsonl`.

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

**Growth**: Appends one entry per conversation start. Averages ~302 bytes/entry (measured
over 14133 entries / 4264783 bytes); the count itself is unbounded and machine-specific.
**Access frequency**: Medium — read at project start.
**Maintenance**: Can be truncated if very large; loses project history but not conversations.

See [`../format/001_history_jsonl.md`](../format/001_history_jsonl.md) for full field spec.

#### .credentials.json — Active API Tokens (~1KB)

**Purpose**: Store active API authentication tokens for Claude Code.
**Format**: Single JSON object with `claudeAiOauth` key.
**Access frequency**: High — read and written on token refresh.
**Security**: High sensitivity. Recommended permissions: `chmod 600 ~/.claude/.credentials.json`

```json
{ "claudeAiOauth": { "... authentication data ..." } }
```

Never delete unless intentionally deauthenticating. Written atomically by `.account.switch`. See [`../format/002_credentials.md`](../format/002_credentials.md) for format spec.

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

#### stats-cache.json — Usage Statistics Cache

**Purpose**: Caches usage statistics and token counts for display in the status bar.
**Format**: JSON object with aggregated usage metrics.
**Access frequency**: Medium — updated during sessions.
**Maintenance**: Safe to delete; will be regenerated from session data.

#### CLAUDE.md — Global Instructions

**Purpose**: User instructions injected into every session regardless of project.
**Format**: Markdown; `@path` references are expanded inline.
**Evidence**: the only `.md` file listed in the binary's own home-maintenance array
alongside `projects` and `settings`-adjacent JSON files (§ Provenance in
[002_support_directories.md](002_support_directories.md)); 17 quoted occurrences.
**Maintenance**: User-owned. Deleting it removes global instructions but breaks nothing.

#### .last-cleanup — Cleanup Sweep Timestamp

**Purpose**: Records when the periodic storage cleanup last ran, so the binary can decide
whether another sweep is due.
**Format**: A bare ISO-8601 timestamp, no JSON wrapper — e.g. `2026-08-27T15:39:52.080Z`.
**Maintenance**: Safe to delete; forces a sweep on next startup.

Directly relevant to `cleanupPeriodDays` — see
[`../param/156_cleanup_period_days.md`](../param/156_cleanup_period_days.md), which
documents that the sweep widened in v2.1.83/v2.1.117 and that `cleanupPeriodDays: 0`
became a validation error in v2.1.89.

#### .last-update-result.json — Self-Update Outcome

**Purpose**: Records the result of the most recent self-update attempt, including failures.
**Format**: Single JSON object. Observed keys: `timestamp`, `path`, `outcome`, `status`,
`version_from`, `version_to`, `error_code`. A real failed-update record:

```json
{"timestamp":"2026-08-08T08:49:31.370Z","path":"native","outcome":"failed",
 "status":"install_failed","version_from":"2.1.197","version_to":null,"error_code":null}
```

**Why it matters**: this is the only persistent record that an auto-update failed. A
machine can sit many versions behind with no other visible signal — check this file first
when installed behavior disagrees with the changelog.
**Maintenance**: Safe to delete; regenerated on the next update attempt.

#### scheduled_tasks.json, launch.json, daemon.json, policy-limits.json

Confirmed root-file names (2–3 quoted occurrences each; all four also appear in the
home-maintenance array or beside it). **Contents not characterized** — none was present on
the surveyed machine, so nothing beyond the names is claimed here. Check for them with
`ls -la ~/.claude/*.json`.

### Not Claude Code's

Two files sit in `~/.claude/` on the surveyed machine and score **0** against the binary
under the same quoted-literal scan that returns 2–17 for every genuine name above, with a
fabricated control also at 0:

| File | Scan result | Verdict |
|------|-------------|---------|
| `cld-timeout-config.json` | 0 | ❌ **Refuted.** A prior revision of this document listed it as a Claude Code root file holding "bash tool timeout configuration". The binary contains no such string. It belongs to separate user tooling; the `cld-` prefix is not a Claude Code convention |
| `settings.json.bak` | 0 | Not created by the binary. `settings.json` is written atomically via `settings.json.tmp` → rename (see below); no `.bak` is produced by that protocol |

Re-check either:

```bash
V=~/.local/share/claude/versions/$(claude --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')
grep -ac '"cld-timeout-config.json"' "$V"   # → 0
grep -ac '"stats-cache.json"' "$V"          # → 1  (positive control)
grep -ac '"NEVER_REAL_FILE.json"' "$V"      # → 0  (negative control)
```

The general lesson: a file's presence in `~/.claude/` is not evidence the `claude` binary
created it. That directory is a shared namespace — plugins, wrappers, and the user all
write into it.

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
| formats | [`../format/001_history_jsonl.md`](../format/001_history_jsonl.md) | history.jsonl entry schema |
| formats | [`../format/002_credentials.md`](../format/002_credentials.md) | .credentials.json structure |
| filesystem | [`../filesystem/001_claude_home.md`](../filesystem/001_claude_home.md) | Path resolution for all `~/.claude/` files |

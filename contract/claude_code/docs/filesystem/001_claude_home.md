# Filesystem: Claude Home

### Scope

- **Purpose**: Document all filesystem paths within the `~/.claude/` configuration root accessed by claude_version.
- **Responsibility**: Authoritative instance for the `~/.claude/` cluster — every path, its access type, the commands that use it, and its purpose.
- **In Scope**: All paths under `~/.claude/` including settings, credentials, profile, projects, sessions, session-env, stats-cache, and downloads.
- **Out of Scope**: Paths outside `~/.claude/` (→ [002_local_install.md](002_local_install.md), [003_credential_store.md](003_credential_store.md), [004_proc_system.md](004_proc_system.md)); storage organization and containment model (→ [`../storage/`](../storage/readme.md)).

### Paths

| Path | Type | Access | Used By | Purpose |
|------|------|--------|---------|---------|
| `~/.claude/` | dir | R | all commands | Configuration root; base for all `ClaudePaths` methods |
| `~/.claude/settings.json` | file | R/W | `.settings.*`, `.version.install`, `.version.guard`, `.status` | User settings; flat JSON with nested object preservation |
| `~/.claude/settings.json.tmp` | file | W | `.settings.set`, `.version.install`, `.version.guard` | Atomic write staging; renamed to `settings.json` on success |
| `~/.claude/.credentials.json` | file | R/W | `.credentials.status`, `.credentials.check`, `.account.save`, `.account.switch` | Active OAuth token; read for status display, overwritten atomically |
| `~/.claude/.claude.json` | file | R | `.credentials.status` | User profile; provides `emailAddress` and `organizationName` |
| `~/.claude/projects/` | dir | R | (reserved) | Conversation history root |
| `~/.claude/sessions/` | dir | R | (reserved) | Session records |
| `~/.claude/session-env/` | dir | R | (reserved) | Per-session environment records |
| `~/.claude/stats-cache.json` | file | R | (reserved) | Usage statistics cache |
| `~/.claude/downloads/` | dir | W | installer (`install.sh`) | Binary staging; installer downloads here before `claude install` |
| `~/.claude/downloads/claude-{ver}-{platform}` | file | W/del | installer | Downloaded binary (temporary); deleted after install |

### Resolution

`~/.claude/` resolves via `ClaudePaths::new()` from the `HOME` environment variable. Returns `None` if `HOME` is unset.

```
~/.claude/ = $HOME/.claude/
```

All sub-paths are derived from this base. The `ClaudePaths` struct in `claude_profile/src/paths.rs` is the authoritative path source — all commands that access `~/.claude/` paths must go through it.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Filesystem master index: full directory tree, path reference table |
| storage | [`../storage/003_root_files.md`](../storage/003_root_files.md) | Root-level files: settings.json, .credentials.json, history.jsonl |
| filesystem | [003_credential_store.md](003_credential_store.md) | Per-account credential files (separate from `~/.claude/.credentials.json`) |
| settings | [`../settings/001_global_settings.md`](../settings/001_global_settings.md) | settings.json write protocol and key table |
| formats | [`../format/002_credentials.md`](../format/002_credentials.md) | `.credentials.json` format: `claudeAiOauth` structure |
| source | `../../../../module/claude_profile/src/paths.rs` | `ClaudePaths` struct — authoritative path implementation |

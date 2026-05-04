# Claude Code: Filesystem Layout

### Scope

- **Purpose**: Centralized reference for all filesystem paths that claude_version reads, writes, or inspects at runtime.
- **Responsibility**: Authoritative directory tree and path reference table for all `~/.claude/`, `~/.local/`, and `{credential_store}` locations.
- **In Scope**: Every filesystem path accessed by claude_version commands and the `/proc/` scanner.
- **Out of Scope**: File format internals and write protocols (→ [005_settings_format.md](005_settings_format.md)); paths internal to the Claude Code binary (npm cache, node internals).

### Directory Tree

```
$HOME/
├── .claude/                            # Claude Code configuration root
│   ├── settings.json                   # User settings (key-value pairs)
│   ├── .credentials.json              # Active OAuth token
│   ├── .claude.json                   # User profile (emailAddress, organizationName)
│   ├── projects/                       # Conversation history root
│   ├── sessions/                       # Session records
│   ├── session-env/                    # Per-session environment records
│   ├── stats-cache.json               # Usage statistics cache
│   └── downloads/                      # Installer staging area (managed by install.sh)
│       └── claude-{ver}-{platform}    # Downloaded binary (temporary)
├── .local/
│   ├── bin/
│   │   └── claude                      # Launcher binary (on $PATH)
│   └── share/
│       └── claude/
│           └── versions/               # Versioned binaries (subject to chmod lock)
└── (system)
    └── /proc/{pid}/                    # Linux process filesystem (read-only)
        ├── cmdline                     # Process command line
        └── cwd → ...                   # Process working directory (symlink)

{credential_store}/                     # clp credential store ($PRO/.persistent/claude/credential/ or $HOME/.persistent/claude/credential/)
├── _active                             # Active account name (plain text)
└── {name}.credentials.json            # Per-account credential file
```

### Path Reference Table

| Path | Type | Access | Used By | Purpose |
|------|------|--------|---------|---------|
| `~/.claude/` | dir | R | all commands | Configuration root; base for all `ClaudePaths` methods |
| `~/.claude/.credentials.json` | file | R/W | `.credentials.status`, `.credentials.check`, `.account.save`, `.account.switch` | Active OAuth token; read for status display, overwritten atomically by `.account.switch` |
| `~/.claude/.claude.json` | file | R | `.credentials.status` | User profile; provides `emailAddress` and `organizationName` for Email: and Org: output lines |
| `~/.claude/settings.json` | file | R/W | `.settings.*`, `.version.install`, `.version.guard`, `.status` | User settings; flat JSON with nested object preservation |
| `~/.claude/settings.json.tmp` | file | W | `.settings.set`, `.version.install`, `.version.guard` | Atomic write staging; renamed to `settings.json` on success |
| `{credential_store}/` | dir | R/W | `.account.*` | Credential store directory; created on first save; absent = no saved accounts |
| `{credential_store}/_active` | file | R/W | `.account.switch`, `.account.status`, `.status` | Active account name (single line, plain text); `{credential_store}` = `$PRO/.persistent/claude/credential/` or `$HOME/.persistent/claude/credential/` |
| `{credential_store}/{name}.credentials.json` | file | R/W/del | `.account.save`, `.account.switch`, `.account.delete`, `.account.list` | Per-account credential snapshot; `name` is an email address |
| `~/.claude/projects/` | dir | R | (reserved) | Conversation history root |
| `~/.claude/sessions/` | dir | R | (reserved) | Session records |
| `~/.claude/session-env/` | dir | R | (reserved) | Per-session environment records |
| `~/.claude/stats-cache.json` | file | R | (reserved) | Usage statistics cache |
| `~/.claude/downloads/` | dir | W | installer (`install.sh`) | Binary staging; installer downloads here before `claude install` |
| `~/.local/bin/claude` | file | R/del | `.version.install`, `.version.guard`, `.version.show` | Launcher binary; resolved via `which claude`, fallback `~/.local/bin/claude` |
| `~/.local/share/claude/versions/` | dir | chmod | `.version.install`, `.version.guard` | Versioned binaries; `chmod 555` (locked) or `755` (unlocked) |
| `/proc/{pid}/cmdline` | file | R | `.processes`, `.processes.kill` | Process command line for Claude process detection |
| `/proc/{pid}/cwd` | symlink | R | `.processes`, `.processes.kill` | Working directory of detected Claude process |

### Path Resolution

| Path | Resolution Method |
|------|-------------------|
| `~/.claude/*` | `ClaudePaths::new()` from `HOME` env var; returns `None` if `HOME` unset |
| `{credential_store}` | `PersistPaths::credential_store()`; resolves to `$PRO/.persistent/claude/credential/` if `PRO` is set, else `$HOME/.persistent/claude/credential/` |
| `~/.local/bin/claude` | `which claude` (preferred); falls back to `$HOME/.local/bin/claude` |
| `~/.local/share/claude/versions` | Hardcoded: `$HOME/.local/share/claude/versions` |
| `/proc/` | Direct filesystem access (Linux only) |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [`005_settings_format.md`](005_settings_format.md) | Atomic write protocol, version lock operations, settings JSON structure |
| doc | [`../../module/claude_version/docs/feature/003_settings_management.md`](../../module/claude_version/docs/feature/003_settings_management.md) | Settings JSON structure feature doc |
| doc | [`../../module/claude_version/docs/feature/001_version_management.md`](../../module/claude_version/docs/feature/001_version_management.md) | Hot-swap and version lock feature doc |
| source | [`../../module/claude_version/src/commands.rs`](../../module/claude_version/src/commands.rs) | `require_claude_paths()`, `hot_swap_binary()`, `versions_dir_path()` |
| source | [`../../module/claude_profile/src/paths.rs`](../../module/claude_profile/src/paths.rs) | `ClaudePaths` struct — authoritative path source |
| source | [`../../module/claude_runner_core/src/process.rs`](../../module/claude_runner_core/src/process.rs) | `/proc` scanner |

# Claude Code: Filesystem Layout

### Scope

- **Purpose**: Centralized reference for all filesystem paths that claude_manager reads, writes, or inspects at runtime.
- **Responsibility**: Authoritative directory tree and path reference table for all `~/.claude/` and `~/.local/` locations.
- **In Scope**: Every filesystem path accessed by claude_manager commands and the `/proc/` scanner.
- **Out of Scope**: File format internals and write protocols (→ [005_settings_format.md](005_settings_format.md)); paths internal to the Claude Code binary (npm cache, node internals).

### Directory Tree

```
$HOME/
├── .claude/                            # Claude Code configuration root
│   ├── settings.json                   # User settings (key-value pairs)
│   ├── .credentials.json              # Active OAuth token
│   ├── accounts/                       # Named credential snapshots
│   │   ├── _active                     # Active account name (plain text)
│   │   └── {name}.credentials.json    # Per-account credential file
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
```

### Path Reference Table

| Path | Type | Access | Used By | Purpose |
|------|------|--------|---------|---------|
| `~/.claude/` | dir | R | all commands | Configuration root; base for all `ClaudePaths` methods |
| `~/.claude/settings.json` | file | R/W | `.settings.*`, `.version.install`, `.version.guard`, `.status` | User settings; flat JSON with nested object preservation |
| `~/.claude/settings.json.tmp` | file | W | `.settings.set`, `.version.install`, `.version.guard` | Atomic write staging; renamed to `settings.json` on success |
| `~/.claude/accounts/_active` | file | R | `.status` | Active account name (single line, plain text) |
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
| `~/.local/bin/claude` | `which claude` (preferred); falls back to `$HOME/.local/bin/claude` |
| `~/.local/share/claude/versions` | Hardcoded: `$HOME/.local/share/claude/versions` |
| `/proc/` | Direct filesystem access (Linux only) |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [`005_settings_format.md`](005_settings_format.md) | Atomic write protocol, version lock operations, settings JSON structure |
| doc | [`../../module/claude_manager/docs/feature/003_settings_management.md`](../../module/claude_manager/docs/feature/003_settings_management.md) | Settings JSON structure feature doc |
| doc | [`../../module/claude_manager/docs/feature/001_version_management.md`](../../module/claude_manager/docs/feature/001_version_management.md) | Hot-swap and version lock feature doc |
| source | [`../../module/claude_manager/src/commands.rs`](../../module/claude_manager/src/commands.rs) | `require_claude_paths()`, `hot_swap_binary()`, `versions_dir_path()` |
| source | [`../../module/claude_profile/src/paths.rs`](../../module/claude_profile/src/paths.rs) | `ClaudePaths` struct — authoritative path source |
| source | [`../../module/claude_runner_core/src/process.rs`](../../module/claude_runner_core/src/process.rs) | `/proc` scanner |

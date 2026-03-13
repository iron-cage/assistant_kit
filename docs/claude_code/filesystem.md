# Filesystem Layout

All directories and files that claude_manager reads, writes, or inspects at runtime.

### Scope

- In scope: every filesystem path accessed by claude_manager commands
- Out of scope: paths internal to the Claude Code binary itself (npm cache, node internals)

### Purpose

Centralized reference for all filesystem locations, preventing path knowledge from scattering across spec, CLI docs, and source comments. For file format internals and write protocols, see [settings_format.md](settings_format.md).

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

- [settings_format.md](settings_format.md) — atomic write protocol, version lock operations, settings JSON structure
- [feature/003_settings_management.md](../../module/claude_manager/docs/feature/003_settings_management.md) — settings JSON structure
- [feature/001_version_management.md](../../module/claude_manager/docs/feature/001_version_management.md) — hot-swap and version lock
- [commands.rs](../src/commands.rs) — `require_claude_paths()`, `hot_swap_binary()`, `versions_dir_path()`
- [paths.rs](../../../claude_profile/src/paths.rs) — `ClaudePaths` struct (authoritative path source)
- [process.rs](../../../claude_runner_core/src/process.rs) — `/proc` scanner (owned by `claude_runner_core`)

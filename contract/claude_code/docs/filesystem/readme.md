# Filesystem

### Scope

- **Purpose**: Centralized reference for all filesystem paths that `claude_version` reads, writes, or inspects at runtime.
- **Responsibility**: Master file for the `filesystem` doc entity — lists all 4 filesystem location cluster instances and provides the authoritative directory tree and path reference table.
- **In Scope**: Every filesystem path accessed by claude_version commands and the `/proc/` scanner: `~/.claude/`, `~/.local/`, `{credential_store}`, and `/proc/{pid}/`.
- **Out of Scope**: File format internals and write protocols (→ [`../settings/`](../settings/readme.md), [`../formats/`](../formats/readme.md)); paths internal to the Claude Code binary (npm cache, node internals); storage directory organization and containment model (→ [`../storage/`](../storage/readme.md)).

### Overview Table

| ID | Name | Responsibility |
|----|------|----------------|
| [001](001_claude_home.md) | Claude Home | `~/.claude/` — configuration root; all settings, session data, and support directories |
| [002](002_local_install.md) | Local Install | `~/.local/bin/claude` (launcher) and `~/.local/share/claude/versions/` (versioned binaries) |
| [003](003_credential_store.md) | Credential Store | `{credential_store}/` — per-account credential snapshots and active account marker |
| [004](004_proc_system.md) | Proc System | `/proc/{pid}/` — Linux process filesystem read for Claude process detection |

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
│   └── downloads/                      # Installer staging area
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
        └── cwd -> ...                  # Process working directory (symlink)

{credential_store}/                     # clp credential store
├── _active                             # Active account name (plain text)
└── {name}.credentials.json            # Per-account credential file
```

`{credential_store}` resolves to `$PRO/.persistent/claude/credential/` when `$PRO` is set, else `$HOME/.persistent/claude/credential/`.

### Path Reference Table

| Path | Type | Access | Used By | Purpose |
|------|------|--------|---------|---------|
| `~/.claude/` | dir | R | all commands | Configuration root; base for all `ClaudePaths` methods |
| `~/.claude/.credentials.json` | file | R/W | `.credentials.status`, `.credentials.check`, `.account.save`, `.account.switch` | Active OAuth token; read for status display, overwritten atomically by `.account.switch` |
| `~/.claude/.claude.json` | file | R | `.credentials.status` | User profile; provides `emailAddress` and `organizationName` |
| `~/.claude/settings.json` | file | R/W | `.settings.*`, `.version.install`, `.version.guard`, `.status` | User settings; flat JSON with nested object preservation |
| `~/.claude/settings.json.tmp` | file | W | `.settings.set`, `.version.install`, `.version.guard` | Atomic write staging; renamed to `settings.json` on success |
| `{credential_store}/` | dir | R/W | `.account.*` | Credential store directory; created on first save |
| `{credential_store}/_active` | file | R/W | `.account.switch`, `.account.status`, `.status` | Active account name (single line, plain text) |
| `{credential_store}/{name}.credentials.json` | file | R/W/del | `.account.save`, `.account.switch`, `.account.delete`, `.account.list` | Per-account credential snapshot |
| `~/.claude/projects/` | dir | R | (reserved) | Conversation history root |
| `~/.claude/sessions/` | dir | R | (reserved) | Session records |
| `~/.claude/session-env/` | dir | R | (reserved) | Per-session environment records |
| `~/.claude/stats-cache.json` | file | R | (reserved) | Usage statistics cache |
| `~/.claude/downloads/` | dir | W | installer (`install.sh`) | Binary staging |
| `~/.local/bin/claude` | file | R/del | `.version.install`, `.version.guard`, `.version.show` | Launcher binary; resolved via `which claude`, fallback `~/.local/bin/claude` |
| `~/.local/share/claude/versions/` | dir | chmod | `.version.install`, `.version.guard` | Versioned binaries; `chmod 555` (locked) or `755` (unlocked) |
| `/proc/{pid}/cmdline` | file | R | `.processes`, `.processes.kill` | Process command line for Claude process detection |
| `/proc/{pid}/cwd` | symlink | R | `.processes`, `.processes.kill` | Working directory of detected Claude process |

### Type-Specific Requirements

All `filesystem` doc instances must include:

1. **Title**: `# Filesystem: {Cluster Name}` — using `Filesystem` as the type prefix
2. **Scope** (H3): 4 required bullets — Purpose, Responsibility, In Scope, Out of Scope
3. **Paths** (H3): Table of all paths in this cluster with Type, Access, Used By, Purpose columns
4. **Resolution** (H3): How paths are resolved (env vars, fallbacks, `ClaudePaths` methods)
5. **Cross-References** (H3): Flat table with `Type | File | Responsibility` columns

### Cross-Doc Entity Dependencies

**This entity depends on**:
- `../settings/` — write protocols for `settings.json` and `settings.json.tmp`
- `../storage/` — `~/.claude/projects/` storage organization

**This entity consumed by**:
- `../../../../module/claude_version/docs/` — version management and settings management features
- `../../../../module/claude_profile/src/paths.rs` — `ClaudePaths` struct implementation
- `../../../../module/claude_runner_core/src/process.rs` — `/proc` scanner

# Format: Shell Snapshot

### Scope

- **Purpose**: Specify the `~/.claude/shell-snapshots/{uuid}.sh` format — bash scripts that preserve and restore shell environment for session resumption.
- **Responsibility**: Authoritative instance for shell snapshot format — script structure, components, encoding, file naming, growth.
- **In Scope**: File location pattern, bash script structure, alias cleanup, base64-encoded function restoration, shell options, file naming convention.
- **Out of Scope**: Shell snapshot directory context (→ [`../storage/002_support_directories.md`](../storage/002_support_directories.md)).

### Location

`~/.claude/shell-snapshots/{session-uuid}.sh`

**Format**: Executable bash script.
**Mutability**: Create-once per session (not modified after creation).

### Schema

```bash
# Snapshot file
# Unset all aliases to avoid conflicts with functions
unalias -a 2>/dev/null || true

# Functions
eval "$(echo 'Z2F3a2xpYnBhdGhfYXBwZW5k...' | base64 -d)"

# Shell options
shopt -s checkwinsize
shopt -s cmdhist
shopt -s expand_aliases
shopt -u histappend
```

### Components

| Component | Purpose |
|-----------|---------|
| Alias cleanup | `unalias -a 2>/dev/null \|\| true` — clears aliases before restoring functions |
| Function restoration | `eval "$(echo 'BASE64' \| base64 -d)"` — restores bash functions (base64-encoded to preserve complex syntax) |
| Shell options | `shopt -s option` / `shopt -u option` — restores shell option state |

### File Naming

`{session-uuid}.sh` — UUID matches the corresponding session ID in `~/.claude/projects/`.

### Growth

- One file per CLI session with shell context
- Size: 5KB–500KB per snapshot (depends on shell complexity)
- Growth: one file per `claude` invocation from a shell environment

### Maintenance

Old snapshots can be deleted safely — only affects ability to restore shell environment for old sessions. Current session's snapshot is needed for active session restoration only.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Formats master index |
| storage | [`../storage/002_support_directories.md`](../storage/002_support_directories.md) | `shell-snapshots/` directory: size, maintenance |

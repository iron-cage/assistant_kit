# Filesystem: Proc System

### Scope

- **Purpose**: Document the `/proc/{pid}/` paths accessed by the claude_version process scanner on Linux.
- **Responsibility**: Authoritative instance for the `/proc/` filesystem cluster — `cmdline` and `cwd` paths; Linux-only applicability; process detection use.
- **In Scope**: `/proc/{pid}/cmdline`, `/proc/{pid}/cwd`; process detection algorithm; Linux-only constraint.
- **Out of Scope**: `~/.claude/` paths (→ [001_claude_home.md](001_claude_home.md)); `~/.local/` paths (→ [002_local_install.md](002_local_install.md)).

### Paths

| Path | Type | Access | Used By | Purpose |
|------|------|--------|---------|---------|
| `/proc/{pid}/cmdline` | file | R | `.processes`, `.processes.kill` | Process command line for Claude process detection |
| `/proc/{pid}/cwd` | symlink | R | `.processes`, `.processes.kill` | Working directory of detected Claude process (symlink to actual path) |

### Resolution

Direct filesystem access — no environment variable lookup. Linux-only; these paths do not exist on macOS or Windows.

```
/proc/ = Linux process filesystem (kernel-provided, read-only from userspace)
/proc/{pid}/ = per-process directory (exists while process is running)
```

### Process Detection

The `.processes` and `.processes.kill` commands scan `/proc/` to detect running Claude Code instances:

1. Enumerate all numeric subdirectories in `/proc/` (each is a PID)
2. Read `/proc/{pid}/cmdline` — null-byte delimited argv; check if the binary path contains "claude"
3. Read `/proc/{pid}/cwd` symlink — resolve to actual working directory path
4. Match against project path encoding to associate process with project

This is Linux-specific functionality. The `/proc/` filesystem is read-only from userspace (no writes).

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Filesystem master index: full directory tree, path reference table |
| source | `../../../../module/claude_runner_core/src/process.rs` | `/proc` scanner implementation |

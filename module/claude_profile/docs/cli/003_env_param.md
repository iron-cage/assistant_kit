# Environment Parameters

clp reads three environment variables for path resolution. No CLI parameter can be set via environment variable — this file documents the input mechanism (naming conventions, precedence, discovery), not per-parameter mappings.

### Naming Convention

clp does not define a custom `CLP_*` namespace. All three variables are OS conventions consumed directly.

### Variables

| Variable | Platform | Purpose | Priority |
|----------|----------|---------|----------|
| `PRO` | All | User-defined project root for persistent storage | 1 — overrides HOME when set to an existing directory |
| `HOME` | Linux / macOS | OS user home directory | 2 — fallback when `PRO` is absent or invalid |
| `USERPROFILE` | Windows | OS user home directory | 2 — fallback (Windows only) |

### Precedence

```
$PRO (set AND points to existing dir) → $HOME / $USERPROFILE → error (exit 2)
```

- `$PRO` is accepted only when **set** and pointing to an **existing directory** on disk. Set-but-invalid (file path, non-existent path) → silently skipped; falls through to step 2.
- `$HOME` / `$USERPROFILE` are the standard OS fallbacks; no extra validation applied.
- If no variable resolves → commands that require paths return exit 2.

### Derived Paths

| Path | Resolution |
|------|------------|
| Credential store | `{root}/.persistent/claude/credential/` |
| Persistent base | `{root}/.persistent/claude_profile/` |
| Live credentials | `{HOME}/.claude/.credentials.json` |
| Claude JSON | `{HOME}/.claude.json` |
| Claude settings | `{HOME}/.claude/settings.json` |

`{root}` = resolved `$PRO` or `$HOME` / `$USERPROFILE`. Live credential paths (`~/.claude/`) always use `$HOME` regardless of `$PRO`.

### Discovery

```bash
# Show all resolved paths including credential store root
clp .paths
# Claude JSON:      /home/user/.claude.json
# Credentials:      /home/user/.claude/.credentials.json
# Settings:         /home/user/.claude/settings.json
# Credential store: /home/user/.persistent/claude/credential/
```

### Source

- Resolution algorithm: [feature/010_persistent_storage.md](../feature/010_persistent_storage.md) (`PersistPaths`)
- HOME-derived paths: [feature/007_file_topology.md](../feature/007_file_topology.md) (`ClaudePaths`)

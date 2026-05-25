# Command :: 1. `.status`

Show Claude Code storage overview and statistics. Use this when you need a global count of projects and sessions, or want to verify the storage root location.

**Parameters:** `path::`, `verbosity::`

**Exit:** `0` success | `1` argument error | `2` storage read error

**Syntax:**
```bash
claude_storage .status
claude_storage .status verbosity::2
claude_storage .status path::/custom/storage verbosity::3
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `path::` | [`StoragePath`](../type/10_storage_path.md) | optional | `~/.claude/` | Storage root override |
| `verbosity::` | [`VerbosityLevel`](../type/12_verbosity_level.md) | optional | `1` | Output detail level |

See [Output Control group](../param_group/01_output_control.md) for `verbosity` semantics.

**Verbosity output levels:**
- `0` — machine-readable counts only (`projects: N, sessions: N`)
- `1` — summary table with project and session totals (default)
- `2` — adds per-project session counts and user/assistant entry breakdowns

**Examples:**
```bash
# Default storage summary
claude_storage .status
# Output: summary table with project/session totals

# Detailed per-project breakdown
claude_storage .status verbosity::2
# Output: summary plus per-project session and entry counts
```

**Notes:**
- Default storage path is `~/.claude/`; override with `CLAUDE_STORAGE_ROOT` env var
- `verbosity::0` is suitable for piping to other tools

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|------------|-----------------|
| 1 | [Output Control](../param_group/01_output_control.md) | Full | — |
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | Partial | `scope::` |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |
| 4 | [Query Storage Programmatically](../user_story/004_query_storage_programmatically.md) | developer |

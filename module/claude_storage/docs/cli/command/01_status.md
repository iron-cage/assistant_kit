# Command :: 1. `.status`

Show Claude Code storage overview and statistics. Use this when you need a global count of projects and sessions, or want to verify the storage root location.

**Parameters:** `path::`, `show_tokens::`

**Exit:** `0` success | `1` argument error | `2` storage read error

**Syntax:**
```bash
claude_storage .status
claude_storage .status show_tokens::1
claude_storage .status path::/custom/storage
```

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `path::` | [`StoragePath`](../type/10_storage_path.md) | optional | `~/.claude/` | Storage root override |
| `show_tokens::` | Boolean | optional | `0` | Show token usage section (triggers full JSONL parse — slow for large storage) |

See [Output Control group](../param_group/01_output_control.md) for output toggle semantics.

**Output:**

Default fast path (filesystem only, completes in < 1 second):
```
Storage: ~/.claude/
Projects: 42 (UUID: 10, Path: 32)
Sessions: 187 (Main: 164, Agent: 23)
```

With `show_tokens::1` (full JSONL parse — may take minutes on large storage):
```
Storage: ~/.claude/
Projects: 42 (UUID: 10, Path: 32)
Sessions: 187 (Main: 164, Agent: 23)
Entries: 8432 (User: 4216, Assistant: 4216)
Tokens:
- Input: 12345678
- Output: 6789012
- Cache Read: 3456789
- Cache Creation: 1234567
```

**Examples:**
```bash
# Default storage summary (fast)
claude_storage .status

# Full stats including token usage (slow — parses all JSONL)
claude_storage .status show_tokens::1
```

**Notes:**
- Default storage path is `~/.claude/`; override with `CLAUDE_STORAGE_ROOT` env var
- `show_tokens::1` triggers full JSONL parsing of all session files — can be slow on large storage

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|------------|-----------------|
| 1 | [Output Control](../param_group/01_output_control.md) | Partial | `show_stat::`, `show_tree::` |
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | Partial | `scope::` |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |

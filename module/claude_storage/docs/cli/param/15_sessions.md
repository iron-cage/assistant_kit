# Parameter :: 15. `sessions::`

Explicit control over session display in `.list`.

**Type:** Boolean

**Fundamental Type:** Boolean flag

**Constraints:**
- Valid values: `0`, `1`
- `0` = suppress session display (even when session filters are active)
- `1` = force session display (even with no session filters)
- Auto-enabled by `session::`, `agent::`, or `min_entries::`

**Default:** `0` (auto-detection active)

**Commands:** `.list`

**Purpose:** Normally session display is auto-controlled: the presence of any session filter enables it. `sessions::` provides an explicit override — `sessions::0` suppresses display even when filters are set (useful for counting projects that have matching sessions), and `sessions::1` forces display even with no filters.

**Examples:**
```bash
sessions::0    # Force off (suppress even when filters active)
sessions::1    # Force on (show even with no filters)
               # (unset) — auto-detect from other params
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Boolean | Base type | Boolean flag | `0` (suppress), `1` (force), or unset (auto) |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 2 | [`.list`](../command/02_list.md) | `0` (auto) | Explicit override for session display; auto-enabled by `session::`, `agent::`, `min_entries::` |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |

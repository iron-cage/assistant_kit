# Parameter :: 15. `show_sessions::` — DEPRECATED

> **Deprecated.** `.list` is deprecated (see [`02_list.md`](../command/02_list.md)); this parameter has no equivalent on `.projects`, which never needed auto-detection. Use [`detail::`](30_detail.md) instead: `detail::projects` (default) shows the terse overview, `detail::sessions` shows session detail — both unconditional. There is no auto-detect mode on `.projects` — `detail::` is always explicit, which is simpler than the auto-enable behavior this parameter provided.

### Scope

- **Purpose**: Specify the `show_sessions::` CLI parameter (deprecated).
- **Responsibility**: Historical type, defaults, valid values, and command usage for `show_sessions::`, retained for traceability.
- **In Scope**: Value constraints, default behavior, command interactions — as they existed before deprecation.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`), current behavior (→ [`detail::`](30_detail.md)).

Explicit control over session display in `.list` (deprecated).

**Type:** Boolean

**Fundamental Type:** Boolean flag

**Constraints:**
- Valid values: `0`, `1`
- `0` = suppress session display (even when session filters are active)
- `1` = force session display (even with no session filters)
- Auto-enabled by `session::`, `agent::`, or `min_entries::`

**Default:** `0` (auto-detection active)

**Commands:** `.list` (deprecated)

**Purpose:** Normally session display is auto-controlled: the presence of any session filter enables it. `show_sessions::` provides an explicit override — `show_sessions::0` suppresses display even when filters are set (useful for counting projects that have matching sessions), and `show_sessions::1` forces display even with no filters.

**Examples:**
```bash
show_sessions::0    # Force off (suppress even when filters active)
show_sessions::1    # Force on (show even with no filters)
               # (unset) — auto-detect from other params
```

**Migration:** `show_sessions::0` → `detail::projects` (now the default, so it can simply be dropped); `show_sessions::1` or unset → `detail::sessions`.

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Boolean | Base type | Boolean flag | `0` (suppress), `1` (force), or unset (auto) |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 2 | [`.list`](../command/02_list.md) | `0` (auto) | DEPRECATED — historical; auto-enabled by `session::`, `agent::`, `min_entries::` |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |

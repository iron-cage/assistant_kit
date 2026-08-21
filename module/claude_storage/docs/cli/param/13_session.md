# Parameter :: 13. `session::`

### Scope

- **Purpose**: Specify the `session::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `session::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Session identifier parameter — acts as substring filter in listing commands, as exact identifier in counting/search commands.

**Type:** [`SessionFilter`](../type/08_session_filter.md) (in `.list`, `.projects`) / [`SessionId`](../type/09_session_id.md) (in `.count`, `.search`)

**Fundamental Type:** String

**Constraints:**
- Non-empty string expected
- In `.list` and `.projects`: case-insensitive substring match against session filename stem
- In `.count` and `.search`: exact match (used to scope to a specific session)

**Default:** unset (no filter / no scope restriction)

**Commands:** `.list` (deprecated), `.count`, `.search`, `.projects`, `.session.path`

**Per-command semantics:**

| Command | Type | Semantics |
|---------|------|-----------|
| `.list` (deprecated) | SessionFilter | Historical substring filter — see `.projects` below |
| `.projects` | SessionFilter | Substring filter — shows sessions whose ID contains this string |
| `.count` | SessionId | Exact scope — counts entries within this specific session |
| `.search` | SessionId | Exact scope — restricts search to this specific session |
| `.session.path` | SessionId | Exact identifier — pure `{storage}/{id}.jsonl` join, no existence check; mutually exclusive with `latest::`/`topic::` |

**Purpose:** Narrows results by session identity. In listing contexts (`.list`, `.projects`), acts as a substring filter for discovery. In counting/search contexts (`.count`, `.search`), acts as an exact scope pin to a specific session.

**Side effect (historical — `.list` only, deprecated):** Auto-enabled `show_sessions::1` in `.list`. No equivalent side effect in `.projects` — session display is never auto-enabled; pass `detail::sessions` explicitly (see [Session Filter group](../param_group/04_session_filter.md)).

**Examples:**
```bash
# Listing: substring filter
session::commit       # Matches -commit.jsonl, auto-commit.jsonl
session::default      # Matches -default_topic.jsonl

# Counting/search: exact scope
.count target::entries project::abc session::-default_topic
.search query::error session::-default_topic
```

**Group (listing context):** [Session Filter](../param_group/04_session_filter.md) — applies to `.list` and `.projects` only where `session::` acts as a substring filter alongside `agent::` and `min_entries::`.

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`SessionFilter`](../type/08_session_filter.md) | String | String | In `.list`/`.projects`: case-insensitive substring |
| [`SessionId`](../type/09_session_id.md) | String (filename stem) | String | In `.count`/`.search`: exact match |

### Referenced Parameter Groups
| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 4 | [Session Filter](../param_group/04_session_filter.md) | Full | `agent::`, `min_entries::` |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 2 | [`.list`](../command/02_list.md) (deprecated) | unset | Historical; auto-enabled `show_sessions::1` |
| 4 | [`.count`](../command/04_count.md) | unset | SessionId: exact scope pin to a specific session |
| 5 | [`.search`](../command/05_search.md) | unset | SessionId: exact scope pin to a specific session |
| 7 | [`.projects`](../command/07_projects.md) | unset | SessionFilter: substring filter |
| 16 | [`.session.path`](../command/16_session_path.md) | unset | SessionId: pure file-path join, no existence check |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |

# Parameter :: 1. `agent::`

### Scope

- **Purpose**: Specify the `agent::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `agent::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Session type filter for listing operations.

**Type:** Boolean

**Fundamental Type:** Boolean flag

**Constraints:**
- Valid values: `0`, `1`, unset
- `0` = main sessions only (no agent sessions)
- `1` = agent sessions only (no main sessions)
- Unset = all session types (no filter)

**Default:** unset (all session types shown)

**Commands:** `.list`, `.projects`

**Purpose:** Distinguishes between main conversation sessions and agent sub-sessions spawned by tool calls. Agent sessions are stored as `agent-*.jsonl` files and have `isSidechain: true`. Use `agent::1` to inspect sub-agent behavior, `agent::0` to see only top-level conversations.

**Side effect:** Auto-enables `show_sessions::1` in `.list` (see [Session Filter group](../param_group/04_session_filter.md)).

**Examples:**
```bash
# Valid values
agent::0       # Main sessions only
agent::1       # Agent sessions only
               # (unset) — all session types

# Invalid values (rejected with error)
agent::2       # Not a boolean: "agent must be 0 or 1"
agent::yes     # Not a boolean: "agent must be 0 or 1"
```

**Group:** [Session Filter](../param_group/04_session_filter.md)

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Boolean | Base type | Boolean flag | `0` (false) or `1` (true); unset allowed |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 2 | [`.list`](../command/02_list.md) | unset | Auto-enables `show_sessions::1`; filters by session type |
| 7 | [`.projects`](../command/07_projects.md) | unset | Filters sessions by type |

### Referenced Parameter Groups
| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 4 | [Session Filter](../param_group/04_session_filter.md) | Full | `session::`, `min_entries::` |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |

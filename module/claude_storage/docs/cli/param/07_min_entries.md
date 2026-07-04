# Parameter :: 7. `min_entries::`

### Scope

- **Purpose**: Specify the `min_entries::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `min_entries::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Filter sessions by minimum entry count threshold.

**Type:** [`EntryCount`](../type/01_entry_count.md)

**Fundamental Type:** Integer wrapper (non-negative)

**Constraints:**
- Must be integer ≥ 0
- Error on non-integer: `"min_entries must be a non-negative integer, got {value}"`
- Error on negative: `"min_entries must be ≥ 0, got {value}"`

**Default:** unset (no minimum)

**Commands:** `.list`, `.projects`

**Purpose:** Excludes sessions with fewer entries than the threshold. Useful for finding substantive conversations (skip one-message sessions) or for performance (only load sessions known to have content).

**Side effect:** Auto-enables `show_sessions::1` in `.list` (see [Session Filter group](../param_group/04_session_filter.md)).

**Examples:**
```bash
# Valid values
min_entries::0    # No minimum (includes all sessions)
min_entries::10   # At least 10 entries
min_entries::100  # Substantial sessions only

# Invalid values
min_entries::-1   # "min_entries must be ≥ 0, got -1"
min_entries::abc  # "min_entries must be a non-negative integer, got abc"
```

**Group:** [Session Filter](../param_group/04_session_filter.md)

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`EntryCount`](../type/01_entry_count.md) | Integer wrapper | Integer (≥0) | Non-negative; negative rejected |

### Referenced Parameter Groups
| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 4 | [Session Filter](../param_group/04_session_filter.md) | Full | `session::`, `agent::` |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 2 | [`.list`](../command/02_list.md) | unset | Auto-enables `show_sessions::1`; excludes sessions below threshold |
| 7 | [`.projects`](../command/07_projects.md) | unset | Excludes sessions below threshold |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |

# Parameter :: 33. `index::`

### Scope

- **Purpose**: Specify the `index::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `index::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Single-message selector for `.show` — narrows whatever set of messages was about to be rendered (a session's full entry list, or the project-overview tail window) down to exactly the one message at this 1-based position. The other half of "total control over... filtering": seeing only one specific message.

**Type:** Integer

**Fundamental Type:** Integer

**Constraints:**
- Must be a positive integer (1-based — matches the existing entry-numbering convention used by `show_entries::1`'s raw list)
- Error on non-positive: `"index must be a positive integer (1-based), got {value}"`
- Error on out-of-range: `"index out of range: {value} ({n} entries)"`, where `{n}` is the count of the in-scope entry set (post-`last::`-windowing in project-overview branches)

**Default:** — (omitted) — every in-scope message is shown, unchanged

**Commands:** [`.show`](../command/03_show.md)

**Purpose:** Applied after any windowing already in effect — in project-overview branches, `index::` counts within the `last::`-sliced window (1 = the first message of that window), not the session's full history; in session-detail branches, `index::` counts within the session's complete entry list. Composes with [`fields::`](32_fields.md) (project specific attributes from just that one message) or stands alone (show that one message's normal chat-log content, or its one raw-list line under `show_entries::1`).

**Examples:**
```bash
# The 3rd message of the session, full chat-log format
.show session_id::abc123 index::3

# The 3rd message's uuid and model only
.show session_id::abc123 fields::uuid,model index::3

# The 1st message of the tail window (project overview)
.show last::10 index::1

# Out of range (session has 5 entries) — errors, does not clamp
.show session_id::abc123 index::99
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Integer | Base type | Integer | Positive (≥ 1); 1-based position within the in-scope entry set |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 3 | [`.show`](../command/03_show.md) | — (all in-scope messages shown) | Counts within the `last::`-windowed slice in project-overview branches, or the full entry list in session-detail branches |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |

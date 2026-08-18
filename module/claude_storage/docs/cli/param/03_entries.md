# Parameter :: 3. `show_entries::`

### Scope

- **Purpose**: Specify the `show_entries::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `show_entries::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Render the currently-displayed entry window as a raw UUID/type/timestamp list instead of formatted content.

**Type:** Boolean

**Fundamental Type:** Boolean flag

**Constraints:**
- Valid values: `0`, `1`
- `0` = formatted content (default)
- `1` = raw entry list (UUID, type, timestamp — no message content)

**Default:** `0`

**Commands:** `.show`

**Purpose:** When enabled, renders entries as a numbered UUID/type/timestamp list instead of formatted conversation content — useful for inspecting session structure or counting messages without loading full content. Effect depends on which `.show` branch is active: in session-detail metadata mode (`show_metadata::1`), it appends the raw list to the metadata block; in session-detail content mode (default, no `show_metadata::1`), it has no effect — content mode always shows full formatted entries regardless; in project-overview branches (no `session_id::`), it renders the `tail::`-windowed message view (see [`tail::`](25_tail.md)) as the same raw list instead of formatted chat.

**Examples:**
```bash
show_entries::0                                    # Formatted content (default)
show_entries::1 show_metadata::1 session_id::ID     # Raw entry list appended to the metadata block
show_entries::1                                     # Project overview: last tail:: messages rendered as a raw list
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Boolean | Base type | Boolean flag | `0` (false) or `1` (true) |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 3 | [`.show`](../command/03_show.md) | `0` | No effect in session-detail content mode; raw entry list in metadata mode or project-overview |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |

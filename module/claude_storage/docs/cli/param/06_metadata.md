# Parameter :: 6. `show_metadata::`

### Scope

- **Purpose**: Specify the `show_metadata::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `show_metadata::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Show session metadata only, suppressing conversation content.

**Type:** Boolean

**Fundamental Type:** Boolean flag

**Constraints:**
- Valid values: `0`, `1`
- `0` = show conversation content (default)
- `1` = show technical metadata only

**Default:** `0`

**Commands:** `.show`

**Purpose:** When enabled, displays session technical metadata (session ID, entry count, first/last timestamps, token usage) without loading or rendering conversation content. Useful for inspecting session characteristics without reading the full content.

**Examples:**
```bash
show_metadata::0    # Show content (default)
show_metadata::1    # Metadata only
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Boolean | Base type | Boolean flag | `0` (false) or `1` (true) |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 3 | [`.show`](../command/03_show.md) | `0` | Shows metadata only when enabled; mutually exclusive with `show_entries::` |

### Referenced Parameter Groups

None.

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |
| 3 | [Export Session for Review](../user_story/003_export_session_for_review.md) | developer |

# Parameter :: 3. `show_entries::`

### Scope

- **Purpose**: Specify the `show_entries::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `show_entries::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Show all individual entries in a session display.

**Type:** Boolean

**Fundamental Type:** Boolean flag

**Constraints:**
- Valid values: `0`, `1`
- `0` = summary view (default)
- `1` = show all entry records

**Default:** `0`

**Commands:** `.show`

**Purpose:** When enabled, lists every entry in the session with UUID, type, and timestamp, rather than showing conversation content. Useful for inspecting session structure or counting messages without loading full content.

**Examples:**
```bash
show_entries::0    # Summary view
show_entries::1    # Full entry listing
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Boolean | Base type | Boolean flag | `0` (false) or `1` (true) |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 3 | [`.show`](../command/03_show.md) | `0` | Lists all entry records when enabled |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |

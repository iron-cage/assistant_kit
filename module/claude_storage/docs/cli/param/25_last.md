# Parameter :: 25. `last::`

### Scope

- **Purpose**: Specify the `last::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `last::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Number of trailing turns to print for `.tail`, or trailing messages shown in `.show`'s project-overview branches. Zero means show all.

**Type:** Integer

**Fundamental Type:** Integer

**Constraints:**
- Must be a non-negative integer
- `0` means no cap (all turns/messages shown)
- Error on negative: `"last must be non-negative"`

**Default:** `4` for `.tail`; `10` for `.show` (context-dependent — see Referenced Commands)

**Commands:** `.tail`, `.show`

**Alias:** `l`

**Purpose:** Caps how many trailing conversation units are printed. Mirrors `limit::`'s "0 = unlimited" convention, applied within a single resolved session rather than to sessions within a project. On `.tail`, counts **turns** — displayable groups of records, not raw JSONL records (see [`../command/12_tail.md`](../command/12_tail.md)'s Turn Grouping). On `.show`'s project-overview branches (no `session_id::`), caps the messages shown from the project's most-recently-active session, beneath the project summary block; no effect when `session_id::` is given (see [`../command/03_show.md`](../command/03_show.md)).

**Unit differs by command:** `.tail`'s unit is a turn; `.show`'s is a raw entry. One assistant response commonly spans several records, so `.tail last::4` and `.show last::4` do not select the same amount of history.

**Examples:**
```bash
# Print the last 4 turns (default)
.tail

# Print the last 10 turns
.tail last::10

# Same, using the alias
.tail l::10

# Print all turns
.tail last::0

# Project overview with the default last 10 messages
.show

# Project overview with the last 25 messages instead
.show last::25
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Integer | Base type | Integer | Non-negative (≥ 0); `0` means no cap |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 3 | [`.show`](../command/03_show.md) | `10` | Caps trailing messages shown from the most-recently-active session in project-overview branches; no effect when `session_id::` given |
| 12 | [`.tail`](../command/12_tail.md) | `4` | Caps trailing **turns** printed — not raw entries |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 6 | [Quick Context Refresh](../user_story/006_quick_context_refresh.md) | developer |

# Parameter :: 25. `tail::`

### Scope

- **Purpose**: Specify the `tail::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `tail::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Number of trailing entries to print for `.tail`. Zero means show all entries.

**Type:** Integer

**Fundamental Type:** Integer

**Constraints:**
- Must be a non-negative integer
- `0` means no cap (all entries shown)
- Error on negative: `"tail must be non-negative"`

**Default:** `4`

**Commands:** `.tail`

**Purpose:** Caps how many trailing conversation entries `.tail` prints. Mirrors `limit::`'s "0 = unlimited" convention, applied to entries within a single resolved session rather than sessions within a project.

**Examples:**
```bash
# Print the last 4 entries (default)
.tail

# Print the last 10 entries
.tail tail::10

# Print all entries
.tail tail::0
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Integer | Base type | Integer | Non-negative (≥ 0); `0` means no cap |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 12 | [`.tail`](../command/12_tail.md) | `4` | Caps trailing entries printed |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 6 | [Quick Context Refresh](../user_story/006_quick_context_refresh.md) | developer |

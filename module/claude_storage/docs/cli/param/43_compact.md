# Parameter :: 43. `compact::`

### Scope

- **Purpose**: Specify the `compact::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `compact::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Print one line per turn — ordinal, age, speaker, and an elided first line — instead of full turn bodies.

**Type:** Boolean

**Fundamental Type:** Boolean

**Constraints:**
- Accepts `1`/`0` (unilang Boolean parsing)
- Absent is equivalent to `compact::0`

**Default:** unset (full turn bodies)

**Commands:** `.tail`

**Alias:** none

**Purpose:** Full bodies answer "what was said"; compact answers "what happened". One line per turn makes a long stretch of history scannable in a single screen, and the leading ordinal is exactly the `index::`-adjacent number needed to jump into any turn with `.show`.

Each row is `ordinal · age · speaker · first line`, held to the same 76-column width as the default layout, with the body flattened to a single line and elided with `…` where it does not fit.

**Interaction with `full::`:** compact wins — `full::1 compact::1` prints compact rows.

**Examples:**
```bash
# One line per turn for the last 4 turns
.tail compact::1

# Scan the last 40 turns at a glance
.tail compact::1 last::40

# The whole session, one line per turn
.tail compact::1 last::0
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Boolean | Base type | Boolean | `1` enables; absent or `0` disables |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 12 | [`.tail`](../command/12_tail.md) | unset | Switches to one-line-per-turn layout |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 6 | [Quick Context Refresh](../user_story/006_quick_context_refresh.md) | developer |

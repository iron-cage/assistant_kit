# Parameter :: 42. `full::`

### Scope

- **Purpose**: Specify the `full::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `full::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Print every body line of every turn instead of folding long turns behind a continuation hint.

**Type:** Boolean

**Fundamental Type:** Boolean

**Constraints:**
- Accepts `1`/`0` (unilang Boolean parsing)
- Absent is equivalent to `full::0`

**Default:** unset (folding active)

**Commands:** `.tail`

**Alias:** none

**Purpose:** `.tail` folds any turn longer than 8 body lines, printing the first 8 and replacing the remainder with `⋯ N more lines · clg .show session_id::… index::…`. That keeps one long answer from pushing the rest of the window off screen, but it is the wrong default when the long turn is precisely what is being read. `full::1` lifts the cap for every turn in the window.

The fold counts **body lines as written**, not as wrapped by the terminal — a single 900-character paragraph is one line and is never folded. `full::1` therefore changes nothing for prose-heavy turns and everything for list-, diff-, or code-heavy ones.

**Interaction with `compact::`:** `compact::1` prints one line per turn regardless, so `full::1` has no effect alongside it — compact wins.

**Examples:**
```bash
# Fold long turns (default)
.tail

# Print every line of the last 4 turns
.tail full::1

# Read one long turn in its entirety
.tail last::1 full::1
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Boolean | Base type | Boolean | `1` enables; absent or `0` disables |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 12 | [`.tail`](../command/12_tail.md) | unset | Disables the 8-line per-turn fold |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 6 | [Quick Context Refresh](../user_story/006_quick_context_refresh.md) | developer |

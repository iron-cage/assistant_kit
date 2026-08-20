# Parameter :: 36. `order::`

### Scope

- **Purpose**: Specify the `order::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `order::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Sort direction applied to [`.table`](../command/14_table.md)'s [`sort::`](35_sort.md) column.

**Type:** String enum

**Fundamental Type:** String

**Constraints:**
- Valid values: `asc`, `desc`
- Case-insensitive on input
- Error on invalid: `"order must be asc|desc, got {value}"`

**Default:** `desc`

**Commands:** [`.table`](../command/14_table.md) — the only command registering this parameter.

**Purpose:** Flips the direction [`sort::`](35_sort.md)'s chosen column is ranked in. `desc` (default) shows the largest/most-recent value first — the natural "what cost the most" reading. `asc` reverses it — smallest first, useful for finding the cheapest rows or, combined with `sort::group`, an alphabetically-ascending listing. Independent of which `sort::` key is chosen; every key supports both directions identically. Introduced for [`.table`](../command/14_table.md) specifically, alongside [`sort::`](35_sort.md) — no other command in this crate exposes a configurable sort direction. Single-command, constrained-value parameter — no dedicated type doc, matching [`depth::`](26_depth.md)'s and [`limit::`](22_limit.md)'s own precedent.

**Examples:**
```bash
# Default: largest total first
.table

# Smallest total first
.table order::asc

# Fewest calls first
.table sort::calls order::asc

# Invalid value
.table order::descending   # "order must be asc|desc, got descending"
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| String enum | Base type | String | `asc`, `desc` |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 14 | [`.table`](../command/14_table.md) | `desc` | Applies to whichever column [`sort::`](35_sort.md) names |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |

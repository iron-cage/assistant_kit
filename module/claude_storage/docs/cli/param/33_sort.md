# Parameter :: 33. `sort::`

### Scope

- **Purpose**: Specify the `sort::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `sort::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Column [`.table`](../command/14_table.md)'s grouped rows are sorted by. Always operates on already-aggregated row totals — sorting happens after [`group::`](32_group.md), never before.

**Type:** String enum

**Fundamental Type:** String

**Constraints:**
- Valid values: `total`, `input`, `output`, `cache`, `max_context`, `calls`, `sessions`, `group`
- Case-insensitive on input
- Error on invalid: `"sort must be total|input|output|cache|max_context|calls|sessions|group, got {value}"`

**Default:** `total`

**Commands:** [`.table`](../command/14_table.md) — the only command registering this parameter.

**Purpose:** Chooses which computed column ranks the rows. `total` (default) is `input + output + cache` combined — the overall cost view. `input`/`output`/`cache` isolate one token category. `max_context` ranks by the largest single call's context window seen in each row — useful for finding sessions/projects that pushed context limits hardest. `calls` ranks by deduplicated assistant-turn count — activity volume independent of token size. `sessions` ranks by how many distinct sessions contributed to a row — only meaningful when [`group::`](32_group.md) is not `session` (every session-grouped row always has exactly 1). `group` sorts lexicographically by the row's own label, for a stable alphabetical listing instead of a magnitude-based one. Introduced for [`.table`](../command/14_table.md) specifically — every other command in this crate that orders output does so by a fixed, non-configurable key (e.g. [`.usage`](../command/13_usage.md) always orders by session mtime). Single-command, constrained-value parameter — no dedicated type doc, matching [`depth::`](26_depth.md)'s and [`limit::`](22_limit.md)'s own precedent.

**Examples:**
```bash
# Default: highest total-token rows first
.table

# Which model/project made the most calls?
.table group::model sort::calls

# Alphabetical listing instead of magnitude-ranked
.table group::project sort::group order::asc

# Invalid value
.table sort::tokens       # "sort must be total|input|output|cache|max_context|calls|sessions|group, got tokens"
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| String enum | Base type | String | `total`, `input`, `output`, `cache`, `max_context`, `calls`, `sessions`, `group` |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 14 | [`.table`](../command/14_table.md) | `total` | Paired with [`order::`](34_order.md) for direction |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |

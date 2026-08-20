# Parameter :: 32. `group::`

### Scope

- **Purpose**: Specify the `group::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `group::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Aggregation dimension for [`.table`](../command/14_table.md)'s grouped result set — which sessions collapse into the same row.

**Type:** String enum

**Fundamental Type:** String

**Constraints:**
- Valid values: `session`, `project`, `model`, `day`
- Case-insensitive on input
- Error on invalid: `"group must be session|project|model|day, got {value}"`

**Default:** `session`

**Commands:** [`.table`](../command/14_table.md) — the only command registering this parameter.

**Purpose:** Selects which field rows are aggregated by. `session` (default) is the finest granularity — one row per session, no summing, closest to [`.usage`](../command/13_usage.md)'s own shape but still sortable/projectable/filterable unlike that fixed command. `project` sums every session under the same recorded `cwd` into one row — the cost-per-project view. `model` sums by recorded model name (`unknown` for sessions with none). `day` sums by `first_timestamp`'s calendar date, UTC as recorded (`unknown` for sessions with no timestamp). Introduced for [`.table`](../command/14_table.md) specifically — no other command in this crate aggregates sessions at all, so no existing parameter's semantics could be reused or extended to cover this. Single-command, constrained-value parameter — no dedicated type doc, matching [`depth::`](26_depth.md)'s and [`limit::`](22_limit.md)'s own precedent of documenting value constraints inline rather than via a separate `type/` file.

**Examples:**
```bash
# Default: one row per session
.table

# Cost per project
.table group::project

# Which model was used most?
.table group::model sort::calls

# Busiest days
.table group::day sort::sessions

# Invalid value
.table group::user        # "group must be session|project|model|day, got user"
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| String enum | Base type | String | `session`, `project`, `model`, `day` |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 14 | [`.table`](../command/14_table.md) | `session` | Aggregation dimension for the whole result set |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |

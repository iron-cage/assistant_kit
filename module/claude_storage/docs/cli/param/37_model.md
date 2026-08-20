# Parameter :: 37. `model::`

### Scope

- **Purpose**: Specify the `model::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `model::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Model-name substring filter for [`.table`](../command/14_table.md), applied at session granularity before [`group::`](34_group.md) aggregation.

**Type:** String

**Fundamental Type:** String

**Constraints:**
- Any non-empty string; no validation beyond that (unlike `group::`/`sort::`/`order::`/`columns::`, there is no fixed value set to reject against)
- Match semantics: case-insensitive substring against each session's recorded `stats.model`, via the same `StringMatcher` mechanism [`.projects`](../command/07_projects.md)'s [`filter::`](29_filter.md) uses against paths (`claude_storage_core/src/filter.rs`) — applied here to a different field, not a shared type. No dedicated type doc: [`filter::`](29_filter.md)'s own [`PathSubstring`](../type/04_path_substring.md) type doc is explicitly scoped to filesystem paths ("Semantically distinct from `StoragePath`... against filesystem paths"), so reusing it here for a model name would misstate that type's own documented scope; `model::` instead documents its constraints inline, matching [`depth::`](26_depth.md)'s and [`limit::`](22_limit.md)'s own single-command precedent
- A session with no recorded model never matches any set filter (absence is not a wildcard)

**Default:** none (no filtering — every session's model is included)

**Commands:** [`.table`](../command/14_table.md) — the only command registering this parameter.

**Purpose:** Narrows the session set contributing to the table to those whose model name contains the given substring — e.g. `model::opus` matches `claude-opus-5`. Applied **before** grouping: a non-matching session is dropped entirely, including from the `percent` column's denominator (see [`14_table.md`](../command/14_table.md)'s Notes) — filtering out a heavy non-matching session raises the surviving rows' percentages, it does not just hide a row. Composes with [`group::`](34_group.md) freely, including `group::model` itself (filtering to one model then grouping by model collapses to at most one row, mainly useful for confirming a single model's own total/percent-of-everything-matched).

**Examples:**
```bash
# Only Opus sessions, still grouped per-session (default group::)
.table model::opus

# Cost of Haiku usage specifically, one summary row
.table model::haiku group::model

# Compose with columns:: for a compact filtered view
.table model::opus columns::group,total,percent

# No match — table has zero data rows, exit 0 (not an error)
.table model::nonexistent-model-xyz
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| String | Base type | String | Case-insensitive substring match against recorded model name |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 14 | [`.table`](../command/14_table.md) | none (unfiltered) | Applied before grouping; affects the `percent` denominator |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |

# Parameter :: 38. `columns::`

### Scope

- **Purpose**: Specify the `columns::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `columns::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Column projection for [`.rollup`](../command/14_rollup.md) — which of 15 columns to print, and in what order. 13 map to a `RollupRow` field the core engine always computes regardless of projection; `rank` and `project` are the two exceptions, synthesized by the CLI — `rank` from each printed row's final position, `project` from the pre-aggregation `session_id → project_label` map (`Fix(BUG-544)`) — instead of read off `RollupRow` (see Constraints).

**Type:** String (comma-separated list)

**Fundamental Type:** String

**Constraints:**
- Comma-separated list of column keys, e.g. `columns::group,total,calls`; whitespace around each entry is trimmed
- Valid keys: `rank`, `group`, `project`, `sessions`, `calls`, `input`, `output`, `cache`, `cache_write`, `cache_read`, `max_context`, `total`, `percent`, `first`, `last`
- `rank` is display-only (each printed row's 1-indexed position after `sort::`/`order::`/`limit::` have all applied) and, like `first`/`last`, has no matching `sort::` value — it cannot itself be sorted on
- `project` is likewise display-only and unsortable: it renders the session's recorded `cwd` under `group::session`, the group label under `group::project`, and `-` under `group::model`/`group::day`, where a row can span many projects (`Fix(BUG-544)`)
- `group`'s header is not a fixed string — it is the active `group::` dimension (`Session`/`Project`/`Model`/`Day`)
- `cache_write`/`cache_read` are `cache`'s two components (`RollupRow.cache_creation`/`RollupRow.cache_read`) exposed separately; `cache_write + cache_read` always equals `cache` for the same row
- Case-insensitive on input
- Order-preserving — columns print left-to-right in the order given, not a fixed canonical order
- Duplicates are not rejected but are not deduplicated either — repeating a key prints it twice
- Error on unknown key: `"unknown column '{value}' — valid: rank|group|project|sessions|calls|input|output|cache|cache_write|cache_read|max_context|total|percent|first|last"`

**Default:** depends on `group::` (`Fix(BUG-544)`) — `group,project,sessions,calls,input,output,cache,max_context,total,percent` under `group::session` (10 of the 15 keys), `group,sessions,calls,input,output,cache,max_context,total,percent` under `group::project`/`model`/`day` (9 of the 15). Either way omits `rank`, `cache_write`, `cache_read`, `first`, `last`, all opt-in only; `project` is opt-in under the three non-session groupings.

**Commands:** [`.rollup`](../command/14_rollup.md) — the only command registering this parameter.

**Purpose:** Every column is always computed internally by `claude_storage_core::rollup::build_rollup()` regardless of projection — `columns::` is a pure display concern, matching [`.usage`](../command/13_usage.md)'s own core/CLI split (that command's core aggregation always populates every field; only the CLI layer's `render_row`/`format_tokens` decide what's printed). The default set favors count/token metrics (including `max_context`, the "window size" metric) and omits the two verbose ISO-8601 timestamp columns (`first`/`last`) that only matter for time-range auditing. A narrower explicit projection (e.g. `columns::group,total`) is useful for compact scripted output; a wider one (adding `first,last`) surfaces the timestamp span each row spans. Single-command, constrained-value parameter — no dedicated type doc, matching [`depth::`](26_depth.md)'s and [`limit::`](22_limit.md)'s own precedent; the 15-key constraint set is documented inline in the table below rather than via a `type/` file, since (unlike [`ScopeValue`](../type/07_scope_value.md)) no other command shares or could plausibly share this exact key set.

**Column keys:**

| Key | Header | In default set? |
|-----|--------|:---:|
| `rank` | `Rank` | — |
| `group` | `Session`/`Project`/`Model`/`Day` (tracks `group::`) | ✓ |
| `project` | `Project` | ✓ under `group::session` only |
| `sessions` | `Sessions` | ✓ |
| `calls` | `Calls` | ✓ |
| `input` | `Input` | ✓ |
| `output` | `Output` | ✓ |
| `cache` | `Cache` | ✓ |
| `cache_write` | `CacheW` | — |
| `cache_read` | `CacheR` | — |
| `max_context` | `MaxCtx` | ✓ |
| `total` | `Total` | ✓ |
| `percent` | `Pct` | ✓ |
| `first` | `First` | — |
| `last` | `Last` | — |

**Examples:**
```bash
# Default under group::session: 10 columns (Group+Project+8 metrics);
# Rank/CacheW/CacheR/First/Last omitted
.rollup

# Compact cost-only view
.rollup columns::group,total,percent

# Add the timestamp span
.rollup columns::group,total,first,last

# Reordered — Total before Group
.rollup columns::total,group

# Leaderboard: rank column plus split cache read/write
.rollup columns::rank,group,total,cache_write,cache_read

# Invalid key
.rollup columns::group,bogus
# "unknown column 'bogus' — valid: rank|group|project|sessions|calls|input|output|cache|cache_write|cache_read|max_context|total|percent|first|last"
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| String (comma list) | Base type | String | 15 valid keys, order-preserving, case-insensitive |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 14 | [`.rollup`](../command/14_rollup.md) | `group::`-dependent default — 10 columns under `session`, 9 otherwise (never `rank`/`cache_write`/`cache_read`/`first`/`last`) | Pure display projection — every column but `rank`/`project` is always computed |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |

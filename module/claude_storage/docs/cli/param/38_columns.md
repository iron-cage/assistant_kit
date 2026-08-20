# Parameter :: 38. `columns::`

### Scope

- **Purpose**: Specify the `columns::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `columns::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Column projection for [`.table`](../command/14_table.md) — which of the 11 always-computed columns to print, and in what order.

**Type:** String (comma-separated list)

**Fundamental Type:** String

**Constraints:**
- Comma-separated list of column keys, e.g. `columns::group,total,calls`; whitespace around each entry is trimmed
- Valid keys: `group`, `sessions`, `calls`, `input`, `output`, `cache`, `max_context`, `total`, `percent`, `first`, `last`
- Case-insensitive on input
- Order-preserving — columns print left-to-right in the order given, not a fixed canonical order
- Duplicates are not rejected but are not deduplicated either — repeating a key prints it twice
- Error on unknown key: `"unknown column '{value}' — valid: group|sessions|calls|input|output|cache|max_context|total|percent|first|last"`

**Default:** `group,sessions,calls,input,output,cache,max_context,total,percent` (9 of the 11 keys — omits `first`/`last`)

**Commands:** [`.table`](../command/14_table.md) — the only command registering this parameter.

**Purpose:** Every column is always computed internally by `claude_storage_core::table::build_table()` regardless of projection — `columns::` is a pure display concern, matching [`.usage`](../command/13_usage.md)'s own core/CLI split (that command's core aggregation always populates every field; only the CLI layer's `render_row`/`format_tokens` decide what's printed). The default set favors count/token metrics (including `max_context`, the "window size" metric) and omits the two verbose ISO-8601 timestamp columns (`first`/`last`) that only matter for time-range auditing. A narrower explicit projection (e.g. `columns::group,total`) is useful for compact scripted output; a wider one (adding `first,last`) surfaces the timestamp span each row spans. Single-command, constrained-value parameter — no dedicated type doc, matching [`depth::`](26_depth.md)'s and [`limit::`](22_limit.md)'s own precedent; the 11-key constraint set is documented inline in the table below rather than via a `type/` file, since (unlike [`ScopeValue`](../type/07_scope_value.md)) no other command shares or could plausibly share this exact key set.

**Column keys:**

| Key | Header | In default set? |
|-----|--------|:---:|
| `group` | `Group` | ✓ |
| `sessions` | `Sessions` | ✓ |
| `calls` | `Calls` | ✓ |
| `input` | `Input` | ✓ |
| `output` | `Output` | ✓ |
| `cache` | `Cache` | ✓ |
| `max_context` | `MaxCtx` | ✓ |
| `total` | `Total` | ✓ |
| `percent` | `Pct` | ✓ |
| `first` | `First` | — |
| `last` | `Last` | — |

**Examples:**
```bash
# Default: 9 columns, First/Last omitted
.table

# Compact cost-only view
.table columns::group,total,percent

# Add the timestamp span
.table columns::group,total,first,last

# Reordered — Total before Group
.table columns::total,group

# Invalid key
.table columns::group,bogus
# "unknown column 'bogus' — valid: group|sessions|calls|input|output|cache|max_context|total|percent|first|last"
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| String (comma list) | Base type | String | 11 valid keys, order-preserving, case-insensitive |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 14 | [`.table`](../command/14_table.md) | 9-column default (no `first`/`last`) | Pure display projection — every column is always computed |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |

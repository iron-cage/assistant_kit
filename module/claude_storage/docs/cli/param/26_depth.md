# Parameter :: 26. `depth::`

### Scope

- **Purpose**: Specify the `depth::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `depth::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Maximum path-component distance a `scope::under`/`relevant`/`around` walk may travel away from `path::` before a candidate project is dropped.

**Type:** Integer

**Fundamental Type:** Integer

**Constraints:**
- Must be a non-negative integer
- `0` means unbounded (no depth cap)
- Error on negative: `"depth must be non-negative"`

**Default:** `3`

**Commands:** [`.usage`](../command/13_usage.md) — the only command registering this parameter.

**Purpose:** Bounds how far a tree-walking `scope::` value (`under`, `relevant`, `around`) may travel from `path::` before a candidate project is excluded, counted in filesystem path components (e.g. `/a/b/c` is depth 3 from `/`). Introduced for [`.usage`](../command/13_usage.md) specifically — the other `scope::`/`path::` implementers (e.g. [`.projects`](../command/07_projects.md)) walk their ancestor/descendant trees uncapped, because they only list sessions (filesystem-cheap). `.usage` must open and parse every candidate session to compute its stats table, a materially higher per-candidate cost that justifies a depth safety-valve `.projects` doesn't need. Ignored when `scope::` is `local` (single project, no walk) or `global` (whole storage, no anchor to measure distance from).

**Examples:**
```bash
# Default depth (3 path components from path::)
.usage scope::under path::/data/repos/yrd_review

# Shallower walk — only direct children/parents
.usage scope::around depth::1

# Unbounded walk
.usage scope::relevant depth::0

# Ignored — scope::local has no tree to walk
.usage scope::local depth::5
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Integer | Base type | Integer | Non-negative (≥ 0); `0` means unbounded |

### Referenced Parameter Groups
| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | Not a formal member — companion to `scope::`/`path::` for [`.usage`](../command/13_usage.md) only | `scope::`, `path::` |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 13 | [`.usage`](../command/13_usage.md) | `3` | Applies to `under`/`relevant`/`around` scopes only |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |

# Parameter :: 12. `scope::`

### Scope

- **Purpose**: Specify the `scope::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `scope::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Discovery scope for session and project operations.

**Type:** [`ScopeValue`](../type/07_scope_value.md)

**Fundamental Type:** String enum wrapper

**Constraints:**
- Valid values: `relevant`, `local`, `under`, `global`, `around`
- Case-insensitive on input
- Error on invalid: `"scope must be relevant|local|under|global|around, got {value}"`

**Default:** varies by command (see table below)

**Commands:** [`.projects`](../command/07_projects.md), [`.list`](../command/02_list.md) (deprecated), [`.count`](../command/04_count.md), [`.search`](../command/05_search.md), [`.show`](../command/03_show.md), [`.export`](../command/06_export.md), [`.usage`](../command/13_usage.md) — all seven implemented, each genuinely narrowing its discovery/search/count/lookup behavior per the semantics below. See the Status column below and [Scope Configuration](../param_group/05_scope_configuration.md) for the full per-command breakdown. `.list`'s role migrates to `.projects`, whose own `scope::` default is `around`, not `.list`'s former `global` — migrating a bare `.list` call requires explicit `scope::global` on `.projects` to preserve the old boundary.

**Purpose:** Controls which projects are searched or counted. `local` is the narrowest (current project only); `global` is the broadest (all projects). `relevant` walks the ancestor chain from cwd upward to `/`; `under` descends into the subtree; `around` combines both for a full neighborhood view — models "what governs this work and what lives under it."

**Per-command semantics (specified design):**

| Command | Default | Scope semantics | Status |
|---------|---------|-----------------|--------|
| `.list` (deprecated) | `global` | Historical: discovery boundary for project listing when `type::` was `all` (default); `type::uuid`/`type::path` ignored it. Superseded by `.projects`, default `around` (see Purpose above for migration) | Deprecated |
| `.count` | `global` | Boundary for `target::projects` and the `target::sessions`-without-`project::` sum; `target::entries`/`target::conversations` and the no-argument cwd-shortcut ignore it | Implemented |
| `.search` | `global` | Boundary for project discovery when `project::` is absent — narrows which projects are searched for `session::`, or searched directly when neither `project::` nor `session::` is given | Implemented |
| `.show` | `local` | Session search scope when `session_id::` given without `project::` (current project + topic variants at the default); no scope used when `session_id::` absent or `project::` given | Implemented |
| `.export` | `local` | Project search boundary for source session lookup when `project::` is absent | Implemented |
| `.projects` | `around` | Session discovery scope (ancestors + current + descendants) | Implemented |
| `.usage` | `local` | Which sessions' stats are aggregated into the usage table | Implemented |

**Examples:**
```bash
# Valid values
scope::local      # Current project only
scope::relevant   # All ancestor projects up to /
scope::under      # All projects under path
scope::global     # All projects in storage
scope::around     # Ancestors + current + descendants (default for .projects)

# Invalid values
scope::all        # "scope must be relevant|local|under|global|around, got all"
```

**Group:** [Scope Configuration](../param_group/05_scope_configuration.md)

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`ScopeValue`](../type/07_scope_value.md) | String enum wrapper | String | `relevant`, `local`, `under`, `global`, `around` |

### Referenced Parameter Groups
| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | Full | `path::` |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 2 | [`.list`](../command/02_list.md) (deprecated) | `global` | Historical discovery boundary; superseded by `.projects` (default `around`, not `global`) |
| 3 | [`.show`](../command/03_show.md) | `local` | Session search scope when `session_id::` given without `project::` — implemented |
| 4 | [`.count`](../command/04_count.md) | `global` | Boundary for `target::projects`/`target::sessions`-without-`project::` — implemented |
| 5 | [`.search`](../command/05_search.md) | `global` | Boundary for project discovery when `project::` is absent — implemented |
| 6 | [`.export`](../command/06_export.md) | `local` | Project search boundary for source session lookup — implemented |
| 7 | [`.projects`](../command/07_projects.md) | `around` | Session discovery scope — implemented |
| 13 | [`.usage`](../command/13_usage.md) | `local` | Usage table aggregation boundary — implemented |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |
| 4 | [Query Storage Programmatically](../user_story/004_query_storage_programmatically.md) | developer |

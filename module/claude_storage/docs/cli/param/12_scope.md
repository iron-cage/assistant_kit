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

**Commands:** `.list`, `.count`, `.search`, `.show`, `.export`, `.projects`

**Purpose:** Controls which projects are searched or counted. `local` is the narrowest (current project only); `global` is the broadest (all projects). `relevant` walks the ancestor chain from cwd upward to `/`; `under` descends into the subtree; `around` combines both for a full neighborhood view — models "what governs this work and what lives under it."

**Per-command semantics:**

| Command | Default | Scope semantics |
|---------|---------|-----------------|
| `.list` | `global` | Discovery boundary for project listing |
| `.count` | `global` | Boundary for what gets counted |
| `.search` | `global` | Boundary for what gets searched |
| `.show` | `local` | Session search scope when `session_id::` given (current project + topic variants); no scope used when `session_id::` absent |
| `.export` | `local` | Project search boundary for source session lookup |
| `.projects` | `around` | Session discovery scope (ancestors + current + descendants) |

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

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 2 | [`.list`](../command/02_list.md) | `global` | Discovery boundary for project listing |
| 3 | [`.show`](../command/03_show.md) | `local` | Session search scope when `session_id::` given |
| 4 | [`.count`](../command/04_count.md) | `global` | Boundary for what gets counted |
| 5 | [`.search`](../command/05_search.md) | `global` | Boundary for what gets searched |
| 6 | [`.export`](../command/06_export.md) | `local` | Project search boundary |
| 7 | [`.projects`](../command/07_projects.md) | `around` | Session discovery scope |

### Referenced Parameter Groups
| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | Full | `path::` |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |
| 4 | [Query Storage Programmatically](../user_story/004_query_storage_programmatically.md) | developer |

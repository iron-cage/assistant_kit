# Parameter :: 24. `show_tree::`

### Scope

- **Purpose**: Specify the `show_tree::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `show_tree::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Show agent sessions tree-indented under their root session.

**Type:** Boolean

**Fundamental Type:** Boolean

**Default:** `0`

**Commands:** `.projects`

**Purpose:** When set to `1`, switches `.projects` session display from compact family-summary format (v1) to tree-indented format — each agent session appears indented under its root session with `├─`/`└─` connectors, full UUID, and per-session entry count. Replaces the former `verbosity::2` behavior.

Default (0): compact format — root session shown with short UUID, mtime, entry count, and inline agent summary `[N agents: breakdown]`.

**Examples:**
```bash
show_tree::0    # Default — compact family summary per root session
show_tree::1    # Tree-indented agents under root sessions
```

**Group:** [Output Control](../param_group/01_output_control.md)

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Boolean | Base type | Boolean flag | `0` (false) or `1` (true) |

### Referenced Parameter Groups
| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 1 | [Output Control](../param_group/01_output_control.md) | Full | `show_stat::`, `show_tokens::` |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 7 | [`.projects`](../command/07_projects.md) | `0` | Tree-indented agent display instead of compact |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |

# Parameter :: 24. `show_tree::`

### Scope

- **Purpose**: Specify the `show_tree::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `show_tree::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Select the tree layout — what is nested depends on [`detail::`](30_detail.md).

**Type:** Boolean

**Fundamental Type:** Boolean

**Default:** `0`

**Commands:** `.projects`

**Purpose:** `show_tree::1` nests the output; what gets nested is whatever the active `detail::` level lists.

Under `detail::projects` (the default), it nests **projects by directory** — shared ancestors become tree nodes drawn with `├─`/`└─` connectors, and single-child runs collapse into one segment (`~/work/src/shared/assistant_kit` renders as a single node, not four). This is what makes the cwd-bucket nature of a "project" visible: one repository entered from five subdirectories is five sibling leaves under a common node. Default (`0`) is the flat recency table, where every row carries its full path.

Under `detail::sessions`, it nests **agent sessions under their root session** (the v2 format) — each agent indented beneath its root with full UUID and per-session entry count, replacing the former `verbosity::2` behavior. Default (`0`) is the compact family summary: root session with short UUID, mtime, entry count, and an inline `[N agents: breakdown]`.

**Examples:**
```bash
show_tree::0                     # Default — flat recency table (one row per project)
show_tree::1                     # Projects nested by directory
detail::sessions show_tree::0    # Compact family summary per root session
detail::sessions show_tree::1    # Tree-indented agents under root sessions
```

**Group:** [Output Control](../param_group/01_output_control.md)

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Boolean | Base type | Boolean flag | `0` (false) or `1` (true) |

### Referenced Parameter Groups
| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 1 | [Output Control](../param_group/01_output_control.md) | Full | `show_stat::`, `show_tokens::`, `show_topic::` |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 7 | [`.projects`](../command/07_projects.md) | `0` | Nests projects by directory (`detail::projects`) or agents under roots (`detail::sessions`) |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |

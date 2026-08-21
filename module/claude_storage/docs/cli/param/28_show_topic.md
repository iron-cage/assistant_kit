# Parameter :: 28. `show_topic::`

### Scope

- **Purpose**: Specify the `show_topic::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `show_topic::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Append each conversation's topic — the first user message text — to its session line.

**Type:** Boolean

**Fundamental Type:** Boolean

**Default:** `0`

**Commands:** `.projects`

**Purpose:** When set to `1`, each root-session line in the compact family view (and each session line in the flat `agent::`-filtered view) gains the conversation's topic: the text of the session's first `user` entry, with newlines flattened to spaces, trimmed, and truncated to 90 characters. Sessions whose transcript has no readable user text show no topic. The tree view (`show_tree::1`) is unaffected, and so is the default `detail::projects` overview — it has no session lines to annotate, so the parameter is a no-op there.

Default (0): session lines are unchanged — short UUID, mtime, entry count only.

The displayed topic is the **conversation topic** (first-message preview) — distinct from the **session topic** (`topic::` parameter), which is the directory-name component of topic-scoped session paths. See the [dictionary](../001_dictionary.md).

**Examples:**
```bash
show_topic::0    # Default — no message text on session lines
show_topic::1    # Append first user message (flattened, max 90 chars)
```

**Group:** [Output Control](../param_group/01_output_control.md)

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Boolean | Base type | Boolean flag | `0` (false) or `1` (true) |

### Referenced Parameter Groups
| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 1 | [Output Control](../param_group/01_output_control.md) | Full | `show_stat::`, `show_tokens::`, `show_tree::` |

### Referenced Commands
| # | Command | Default | Notes |
|---|--------|---------|-------|
| 7 | [`.projects`](../command/07_projects.md) | `0` | First user message appended to session lines |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |

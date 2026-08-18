# Parameter :: 22. `limit::`

### Scope

- **Purpose**: Specify the `limit::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `limit::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Maximum number of sessions to display. Zero means unlimited. Scope of the cap (per-project vs. flat across the whole result set) is command-dependent — see the table below.

**Type:** Integer

**Fundamental Type:** Integer

**Constraints:**
- Must be a non-negative integer
- `0` means no cap (all sessions shown)
- Error on negative: `"limit must be non-negative"`

**Default:** `0` (unlimited)

**Commands:** `.projects`, [`.usage`](../command/13_usage.md)

**Purpose:** Caps how many sessions are shown, most-recent-first. In `.projects`, the cap applies **per project** in the default view — useful when a project has many sessions and you only want a preview; does not apply in `show_tree::1` mode. In `.usage`, the cap applies **flat across the whole result set** (after `scope::`/`depth::` filtering, before rendering) — there is no per-project grouping to cap within.

**Examples:**
```bash
# Show at most 5 sessions per project
.projects limit::5

# No cap (default)
.projects limit::0

# Combined with scope
.projects scope::global limit::3

# .usage: cap the flat result set to 20 most-recent sessions
.usage scope::global limit::20
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Integer | Base type | Integer | Non-negative (≥ 0); `0` means no cap |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 7 | [`.projects`](../command/07_projects.md) | `0` | Caps sessions per project in default display |
| 13 | [`.usage`](../command/13_usage.md) | `0` | Caps the flat result set (not per-project) |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |

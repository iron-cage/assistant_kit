# Parameter :: 27. `since_days::`

### Scope

- **Purpose**: Specify the `since_days::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `since_days::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Recency window in days: only sessions whose file was modified within the last N days are listed.

**Type:** Integer

**Fundamental Type:** Integer

**Constraints:**
- Must be a non-negative integer
- `0` means the most recent 24 hours (same window as `1` — never an empty window)
- Error on negative: `"Invalid since_days: N. Must be non-negative"`

**Default:** unset (no window — sessions of any age are listed)

**Commands:** `.projects`

**Purpose:** Windows the session listing by file modification time — the same mtime the recency sort already uses. For `N ≥ 1` the cutoff is exactly `now - N × 24h`; `0` is treated as `1` so a session touched today always survives a zero-day window. Sessions whose mtime cannot be read are excluded (they cannot be proven recent). The filter applies before project aggregation, so a project whose sessions all fall outside the window disappears from the listing entirely.

**Examples:**
```bash
# Sessions active in the last 20 days
.projects since_days::20

# Only sessions touched in the last 24 hours
.projects since_days::0

# Combined with global scope and topics
.projects scope::global since_days::20 show_topic::1
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Integer | Base type | Integer | Non-negative (≥ 0); `0` means last 24 hours |

### Referenced Commands
| # | Command | Default | Notes |
|---|--------|---------|-------|
| 7 | [`.projects`](../command/07_projects.md) | unset | Day-window filter on session mtime before aggregation |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |

# Parameter :: 30. `detail::`

### Scope

- **Purpose**: Specify the `detail::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `detail::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Output verbosity selector — terse project overview, or full session detail. On `.projects`, absorbed `.list`'s former project-only default view and its `show_sessions::` toggle into a single explicit parameter (see [`02_list.md`](../command/02_list.md), [`15_sessions.md`](15_sessions.md)). On `.show`'s project-overview branches, gates the full per-session list beneath the project summary block.

**Type:** [`DetailLevel`](../type/14_detail_level.md)

**Fundamental Type:** String enum wrapper

**Constraints:**
- Valid values: `projects`, `sessions`
- Case-insensitive on input
- Error on invalid: `"detail must be projects|sessions, got {value}"`

**Default:** `projects` for both `.projects` and `.show` (context-dependent — see Referenced Commands)

`.projects` defaulted to `sessions` while `.list` was being absorbed, purely to keep bare `.projects` behaving as it had pre-consolidation. That rationale expired with `.list`: a command named `.projects` should answer "which projects?", and on a real machine the session-detail default expands that into thousands of lines. `detail::sessions` is now the opt-in.

**Commands:** [`.projects`](../command/07_projects.md), [`.show`](../command/03_show.md)

**Purpose:** On `.projects`, `detail::projects` (the default) renders the terse overview — one line per project carrying recency, conversation count, agent count, and path, under a totals summary line. [`show_tree::`](24_show_tree.md) picks the layout: flat recency table by default, directory tree at `show_tree::1`. `detail::sessions` is full family/tree session detail beneath a `Found N projects:` header, one block per project. Session filters (`session::`, `agent::`, `min_entries::`) do not auto-enable session lines — they narrow the counts under `detail::projects` and the visible lines under `detail::sessions`; pass `detail::sessions` explicitly to see individual sessions. On `.show`'s project-overview branches (no `session_id::`), the same two values gate the full per-session list beneath the summary block and `last::`-windowed messages — `detail::projects` (also the default there) omits it, `detail::sessions` appends it; no effect when `session_id::` is given.

**Examples:**
```bash
# Terse overview (default) — one line per project
.projects

# Same rows, nested by directory
.projects show_tree::1

# Full session detail beneath each project
.projects detail::sessions

# Terse overview narrowed by scope and filter
.projects scope::global filter::assistant

# Project overview, summary + tail messages only (default on .show)
.show

# Project overview, also list every session
.show detail::sessions
```

**Group:** None — like `filter::`, `type::`, `ids::`, and `count::`, this is a `.projects`-only parameter with no cross-command pattern to document via a group. Not [Output Control](../param_group/01_output_control.md): that group's own "Why NOT `show_sessions::`" rationale excludes tier-visibility toggles (show/hide the whole session block) as a different semantic level from optional-block toggles (add one extra line within an existing block) — `detail::` is `show_sessions::`'s direct successor and is a tier-visibility toggle, so the same exclusion applies to it.

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`DetailLevel`](../type/14_detail_level.md) | String enum wrapper | String | `projects`, `sessions` |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 3 | [`.show`](../command/03_show.md) | `projects` | Gates the full per-session list in project-overview branches; no effect when `session_id::` given |
| 7 | [`.projects`](../command/07_projects.md) | `projects` | Terse overview; `show_tree::` picks flat table vs directory tree |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |

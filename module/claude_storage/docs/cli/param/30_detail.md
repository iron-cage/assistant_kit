# Parameter :: 30. `detail::`

### Scope

- **Purpose**: Specify the `detail::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `detail::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Output verbosity selector — terse project/summary headers only, or full session detail. On `.projects`, absorbed `.list`'s former project-only default view and its `show_sessions::` toggle into a single explicit parameter (see [`02_list.md`](../command/02_list.md), [`15_sessions.md`](15_sessions.md)). On `.show`'s project-overview branches, gates the full per-session list beneath the project summary block.

**Type:** [`DetailLevel`](../type/14_detail_level.md)

**Fundamental Type:** String enum wrapper

**Constraints:**
- Valid values: `projects`, `sessions`
- Case-insensitive on input
- Error on invalid: `"detail must be projects|sessions, got {value}"`

**Default:** `sessions` for `.projects` (preserves its pre-consolidation behavior — bare invocation always showed session detail); `projects` for `.show` (context-dependent — see Referenced Commands)

**Commands:** [`.projects`](../command/07_projects.md), [`.show`](../command/03_show.md)

**Purpose:** `detail::projects` gives the terse, project-only view `.list`'s bare invocation used to provide — one header line per project, no session or family lines. `detail::sessions` is full family/tree session detail beneath each project header (`.projects`' unchanged pre-consolidation behavior, and its default). On `.projects`, because `sessions` is already the default, no auto-enable logic is needed — passing a session filter (`session::`, `agent::`, `min_entries::`) just narrows what the already-visible session lines show; use `detail::projects` explicitly to suppress session lines regardless of filters. On `.show`'s project-overview branches (no `session_id::`), the same two values gate the full per-session list beneath the summary block and `last::`-windowed messages — `detail::projects` (the default here) omits it, `detail::sessions` appends it; no effect when `session_id::` is given.

**Examples:**
```bash
# Full detail (default) — unchanged from pre-consolidation .projects
.projects

# Terse project-only view — replaces bare `.list`
.projects detail::projects

# Terse view narrowed by scope and filter
.projects scope::global filter::assistant detail::projects

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
| 7 | [`.projects`](../command/07_projects.md) | `sessions` | Absorbed from `.list`'s default view and `show_sessions::` |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |

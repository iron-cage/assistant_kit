# Parameter :: 29. `filter::`

### Scope

- **Purpose**: Specify the `filter::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `filter::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Path-substring filter on projects already resolved by `scope::`/`path::`. Absorbed from `.list`'s former `path::` role (see [`../command/02_list.md`](../command/02_list.md)) — kept as a distinct name on `.projects` because that command's own `path::` already means something else (the scope anchor).

**Type:** [`PathSubstring`](../type/04_path_substring.md)

**Fundamental Type:** String

**Constraints:**
- Case-insensitive substring match against each resolved project's decoded display path
- No match syntax beyond plain substring (no globs, no regex)

**Default:** — (no filtering)

**Commands:** [`.projects`](../command/07_projects.md)

**Purpose:** Narrows the project set `scope::` already resolved, the same way `.list`'s `path::` narrowed its own project listing. Composes with `scope::`/`path::` rather than replacing them — `scope::` decides the discovery boundary, `filter::` further narrows by substring within that boundary. Independent of `type::` (naming-scheme filter) and `detail::` (output verbosity) — all three narrow or shape the same resolved set without interacting with each other.

**Examples:**
```bash
# Only projects with "assistant" in their decoded path, within the default scope
.projects filter::assistant

# Combine with a wider scope boundary
.projects scope::global filter::my-app

# Combine with type:: (naming-scheme filter) — both apply
.projects scope::global type::path filter::my-app
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`PathSubstring`](../type/04_path_substring.md) | String | String | Case-insensitive substring match on decoded project path |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 7 | [`.projects`](../command/07_projects.md) | — (no filtering) | Absorbed from `.list`'s former `path::` role |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |

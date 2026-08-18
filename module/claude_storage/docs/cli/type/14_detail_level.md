# Type :: 14. `DetailLevel`

### Scope

- **Purpose**: Specify the `DetailLevel` semantic type.
- **Responsibility**: Validation rules, fundamental type, and parameter mapping for `DetailLevel`.
- **In Scope**: Parsing rules, valid/invalid values, error messages.
- **Out of Scope**: Parameter usage (→ `param/`), command context (→ `command/`).

**Purpose:** Output verbosity selector — terse vs full detail. On `.projects`: project headers only vs project headers plus session/family detail; introduced when `.projects` absorbed `.list`'s project-only view (see [`02_list.md`](../command/02_list.md)). On `.show`'s project-overview branches: summary block only vs summary block plus the full per-session list.

**Fundamental Type:** Wrapper around string enum

**Constants:**
- PROJECTS = `"projects"` (header line only, no session/family lines)
- SESSIONS = `"sessions"` (header line plus full session/family detail — default, matches pre-consolidation `.projects` behavior)
- DEFAULT = SESSIONS

**Constraints:**
- Valid values: `projects`, `sessions`
- Case-insensitive on parse
- Error on invalid: `"detail must be projects|sessions, got {value}"`

**Parsing:**
```
Parse string to enum variant (case-insensitive):
  Input: "projects" → DetailLevel::Projects
  Input: "sessions" → DetailLevel::Sessions
  Error: "detail must be projects|sessions, got {value}"
```

**Methods:**
- `get() -> string` — Canonical lowercase variant name
- `is_projects() -> boolean` — True when terse (header-only) view selected

**Commands:** [`.projects`](../command/07_projects.md), [`.show`](../command/03_show.md)

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 3 | [`.show`](../command/03_show.md) | `detail::` |
| 7 | [`.projects`](../command/07_projects.md) | `detail::` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 30 | [`detail::`](../param/30_detail.md) | 1 |

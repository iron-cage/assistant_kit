# Type :: 11. `TargetType`

**Purpose:** Selects the granularity for `.count` operations. Determines whether to count projects, sessions within a project, or entries within a session.

**Fundamental Type:** Wrapper around string enum

**Constants:**
- PROJECTS = `"projects"` (default)
- SESSIONS = `"sessions"`
- ENTRIES = `"entries"`
- DEFAULT = PROJECTS

**Constraints:**
- Valid values: `projects`, `sessions`, `entries`
- Case-insensitive on parse
- Error on invalid: `"target must be projects|sessions|entries, got {value}"`

**Parsing:**
```
Parse string to enum variant (case-insensitive):
  Input: "projects" → TargetType::Projects
  Input: "sessions" → TargetType::Sessions
  Input: "entries"  → TargetType::Entries
  Error: "target must be projects|sessions|entries, got {value}"
```

**Methods:**
- `get() -> string` — Canonical lowercase variant name
- `is_default() -> boolean` — True when target is Projects
- `requires_project() -> boolean` — True for Sessions and Entries
- `requires_session() -> boolean` — True for Entries only

**Commands:** `.count`

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 4 | [`.count`](../command/04_count.md) | `target::` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 16 | [`target::`](../param/16_target.md) | 1 |

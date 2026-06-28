# Type :: 11. `TargetType`

### Scope

- **Purpose**: Specify the `TargetType` semantic type.
- **Responsibility**: Validation rules, fundamental type, and parameter mapping for `TargetType`.
- **In Scope**: Parsing rules, valid/invalid values, error messages.
- **Out of Scope**: Parameter usage (→ `param/`), command context (→ `command/`).

**Purpose:** Selects the granularity for `.count` operations. Determines whether to count projects, sessions within a project, entries within a session, or conversations within a project.

**Fundamental Type:** Wrapper around string enum

**Constants:**
- PROJECTS = `"projects"` (default)
- SESSIONS = `"sessions"`
- ENTRIES = `"entries"`
- CONVERSATIONS = `"conversations"`
- DEFAULT = PROJECTS

**Constraints:**
- Valid values: `projects`, `sessions`, `entries`, `conversations`
- Case-insensitive on parse
- Error on invalid: `"target must be projects|sessions|entries|conversations, got {value}"`

**Parsing:**
```
Parse string to enum variant (case-insensitive):
  Input: "projects"      → TargetType::Projects
  Input: "sessions"      → TargetType::Sessions
  Input: "entries"       → TargetType::Entries
  Input: "conversations" → TargetType::Conversations
  Error: "target must be projects|sessions|entries|conversations, got {value}"
```

**Methods:**
- `get() -> string` — Canonical lowercase variant name
- `is_default() -> boolean` — True when target is Projects
- `requires_project() -> boolean` — True for Sessions, Entries, and Conversations
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

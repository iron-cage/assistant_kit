# Type :: 8. `SessionFilter`

### Scope

- **Purpose**: Specify the `SessionFilter` semantic type.
- **Responsibility**: Validation rules, fundamental type, and parameter mapping for `SessionFilter`.
- **In Scope**: Parsing rules, valid/invalid values, error messages.
- **Out of Scope**: Parameter usage (→ `param/`), command context (→ `command/`).

**Purpose:** Case-insensitive substring matcher against session IDs (JSONL filename stems). Semantically distinct from `SessionId` — this is a pattern for filtering, not a direct identifier.

**Fundamental Type:** Wrapper around string

**Constants:**
- DEFAULT = unset (no filter applied)

**Constraints:**
- Non-empty string when provided
- Match semantics: case-insensitive substring of session filename stem (without `.jsonl`)

**Parsing:**
```
Validate non-empty string:
  Input: "commit" → SessionFilter("commit")
  Input: ""       → Error("session filter must be non-empty")
```

**Methods:**
- `get() -> string` — Raw substring value
- `matches(session_id: string) -> boolean` — True if session ID contains substring (case-insensitive)

**Commands:** `.list`, `.count`, `.search`, `.projects`

**Usage:**
```
session::commit
# Matches: -commit.jsonl, auto-commit.jsonl, pre-commit.jsonl
# Does not match: -default_topic.jsonl
```

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 2 | [`.list`](../command/02_list.md) | `session::` |
| 4 | [`.count`](../command/04_count.md) | `session::` |
| 5 | [`.search`](../command/05_search.md) | `session::` |
| 7 | [`.projects`](../command/07_projects.md) | `session::` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 13 | [`session::`](../param/13_session.md) | 4 |

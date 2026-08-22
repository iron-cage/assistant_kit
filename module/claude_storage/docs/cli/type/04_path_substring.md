# Type :: 4. `PathSubstring`

### Scope

- **Purpose**: Specify the `PathSubstring` semantic type.
- **Responsibility**: Validation rules, fundamental type, and parameter mapping for `PathSubstring`.
- **In Scope**: Parsing rules, valid/invalid values, error messages.
- **Out of Scope**: Parameter usage (→ `param/`), command context (→ `command/`).

**Purpose:** Case-insensitive substring matcher against filesystem paths. Semantically distinct from `StoragePath` — this is a filter expression, not a filesystem location.

**Fundamental Type:** Wrapper around string

**Constants:**
- DEFAULT = unset (no filter applied)

**Constraints:**
- Non-empty string when provided
- Match semantics: case-insensitive substring of the full filesystem path

**Parsing:**
```
Validate non-empty string:
  Input: "myproject" → PathSubstring("myproject")
  Input: ""          → Error("path filter must be non-empty")
```

**Methods:**
- `get() -> string` — Raw substring value
- `matches(path: string) -> boolean` — True if path contains substring (case-insensitive)

**Commands:** [`.projects`](../command/07_projects.md) (via `filter::`; absorbed from `.list`'s former `path::` role, see [`../command/02_list.md`](../command/02_list.md))

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 7 | [`.projects`](../command/07_projects.md) | `filter::` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 29 | [`filter::`](../param/29_filter.md) | 1 |

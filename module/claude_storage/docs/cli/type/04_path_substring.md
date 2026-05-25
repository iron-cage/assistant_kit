# Type :: 4. `PathSubstring`

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

**Commands:** `.list` (via `path::`)

**Usage:**
```
.list path::assistant
# Matches: /home/alice/projects/assistant/module/core
# Matches: /home/alice/projects/assistant
# Does not match: /home/alice/projects/claude-storage
```

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 2 | [`.list`](../command/02_list.md) | `path::` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 9 | [`path::`](../param/09_path.md) | 11 |

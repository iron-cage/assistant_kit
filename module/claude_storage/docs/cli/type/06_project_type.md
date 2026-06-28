# Type :: 6. `ProjectType`

### Scope

- **Purpose**: Specify the `ProjectType` semantic type.
- **Responsibility**: Validation rules, fundamental type, and parameter mapping for `ProjectType`.
- **In Scope**: Parsing rules, valid/invalid values, error messages.
- **Out of Scope**: Parameter usage (→ `param/`), command context (→ `command/`).

**Purpose:** Filter for project naming scheme. Claude Code names projects either by path-encoding their filesystem path or by UUID.

**Fundamental Type:** Wrapper around string enum

**Constants:**
- PATH = `"path"` (path-encoded projects)
- UUID = `"uuid"` (UUID-named projects)
- ALL = `"all"` (default — no filter)
- DEFAULT = ALL

**Constraints:**
- Valid values: `uuid`, `path`, `all`
- Case-insensitive on parse
- Error on invalid: `"type must be uuid|path|all, got {value}"`

**Parsing:**
```
Parse string to enum variant (case-insensitive):
  Input: "path" → ProjectType::Path
  Input: "uuid" → ProjectType::Uuid
  Input: "all"  → ProjectType::All
  Error: "type must be uuid|path|all, got {value}"
```

**Methods:**
- `get() -> string` — Canonical lowercase variant name
- `is_all() -> boolean` — True when no filter applied
- `matches(project: &Project) -> boolean` — True when project naming matches type

**Commands:** `.list`

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 2 | [`.list`](../command/02_list.md) | `type::` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 18 | [`type::`](../param/18_type.md) | 1 |

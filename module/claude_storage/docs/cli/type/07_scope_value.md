# Type :: 7. `ScopeValue`

### Scope

- **Purpose**: Specify the `ScopeValue` semantic type.
- **Responsibility**: Validation rules, fundamental type, and parameter mapping for `ScopeValue`.
- **In Scope**: Parsing rules, valid/invalid values, error messages.
- **Out of Scope**: Parameter usage (→ `param/`), command context (→ `command/`).

**Purpose:** Controls the discovery boundary for session listing in `.projects`. Defines how broadly to search for matching projects relative to a filesystem anchor path.

**Fundamental Type:** Wrapper around string enum

**Constants:**
- LOCAL = `"local"` (current project only)
- RELEVANT = `"relevant"` (ancestor chain up to `/`)
- UNDER = `"under"` (descendant subtree)
- GLOBAL = `"global"` (all projects)
- AROUND = `"around"` (ancestors + current + descendants — bidirectional) **(default)**
- DEFAULT = AROUND

**Constraints:**
- Valid values: `relevant`, `local`, `under`, `global`, `around`
- Case-insensitive on parse
- Error on invalid: `"scope must be relevant|local|under|global|around, got {value}"`

**Parsing:**
```
Parse string to enum variant (case-insensitive):
  Input: "local"    → ScopeValue::Local
  Input: "relevant" → ScopeValue::Relevant
  Input: "under"    → ScopeValue::Under
  Input: "global"   → ScopeValue::Global
  Input: "around"   → ScopeValue::Around
  Error: "scope must be relevant|local|under|global|around, got {value}"
```

**Methods:**
- `get() -> string` — Canonical lowercase variant name
- `is_default() -> boolean` — True when scope is Around
- `requires_path() -> boolean` — True for Under and Around scopes (path:: optional anchor)
- `ignores_path() -> boolean` — True for Global scope

**Scope comparison:**

| Variant | Direction | Breadth | Composition |
|---------|-----------|---------|-------------|
| `local` | — | 1 project | Exact match of CWD only |
| `relevant` | Up ↑ (ancestors) | N projects | Ancestor walk from CWD to `/` |
| `under` | Down ↓ (descendants) | N projects | Subtree rooted at CWD |
| `around` | Bidirectional ↑↓ | N projects | `relevant` ∪ `under` (deduplicated) |
| `global` | — | All projects | All projects regardless of path |

**`around` semantics:** Union of `relevant` and `under` with deduplication. Ancestor results listed first (CWD → `/`), then descendant results below CWD. Projects appearing in both (including CWD itself) appear once. Models the "project neighborhood" — what governs this work and what lives under it.

**Commands:** `.list`, `.show`, `.count`, `.search`, `.export`, `.projects`

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 2 | [`.list`](../command/02_list.md) | `scope::` |
| 3 | [`.show`](../command/03_show.md) | `scope::` |
| 4 | [`.count`](../command/04_count.md) | `scope::` |
| 5 | [`.search`](../command/05_search.md) | `scope::` |
| 6 | [`.export`](../command/06_export.md) | `scope::` |
| 7 | [`.projects`](../command/07_projects.md) | `scope::` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 12 | [`scope::`](../param/12_scope.md) | 6 |

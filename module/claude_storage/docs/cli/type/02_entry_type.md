# Type :: 2. `EntryType`

### Scope

- **Purpose**: Specify the `EntryType` semantic type.
- **Responsibility**: Validation rules, fundamental type, and parameter mapping for `EntryType`.
- **In Scope**: Parsing rules, valid/invalid values, error messages.
- **Out of Scope**: Parameter usage (→ `param/`), command context (→ `command/`).

**Purpose:** Semantic type representing which conversation participant authored an entry. Restricts search to user messages, assistant messages, or both.

**Fundamental Type:** Wrapper around string enum

**Constants:**
- USER = `"user"`
- ASSISTANT = `"assistant"`
- ALL = `"all"` (default — no filter)
- DEFAULT = ALL

**Constraints:**
- Valid values: `user`, `assistant`, `all`
- Case-insensitive on parse
- Error on invalid: `"entry_type must be user|assistant|all, got {value}"`

**Parsing:**
```
Parse string to enum variant (case-insensitive):
  Input: "user", "User", "USER" → EntryType::User
  Input: "assistant" → EntryType::Assistant
  Input: "all" → EntryType::All
  Error: "entry_type must be user|assistant|all, got {value}"

Pseudocode:
  function parse_entry_type(input: string) -> Result<EntryType>:
    match input.to_lowercase():
      "user"      → EntryType::User
      "assistant" → EntryType::Assistant
      "all"       → EntryType::All
      other       → Error("entry_type must be user|assistant|all, got " + input)
```

**Methods:**
- `get() -> string` — Returns canonical lowercase variant name
- `is_all() -> boolean` — True when no filter applied
- `matches(entry: &Entry) -> boolean` — True when entry type matches this filter

**Commands:** `.search`

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 5 | [`.search`](../command/05_search.md) | `entry_type::` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 4 | [`entry_type::`](../param/04_entry_type.md) | 1 |

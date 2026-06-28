# Type :: 9. `SessionId`

### Scope

- **Purpose**: Specify the `SessionId` semantic type.
- **Responsibility**: Validation rules, fundamental type, and parameter mapping for `SessionId`.
- **In Scope**: Parsing rules, valid/invalid values, error messages.
- **Out of Scope**: Parameter usage (→ `param/`), command context (→ `command/`).

**Purpose:** Direct session identifier by filename stem. Used for single-session operations where the exact session is known.

**Fundamental Type:** Wrapper around string

**Constants:**
- DEFAULT = unset (optional in `.show`; required in `.export`)

**Constraints:**
- Non-empty string
- Error if session not found: `"session not found: {value}"`

**Parsing:**
```
Validate non-empty string, then resolve:
  Input: "-default_topic"                       → SessionId("-default_topic")
  Input: "8d795a1c-c81d-4010-8d29-b4e678272419" → SessionId("8d795a1c-...")
  Input: ""                                     → Error("session_id must be non-empty")
  Resolve → Error("session not found: " + input) if file not in storage
```

**Methods:**
- `get() -> string` — Raw session ID value
- `filename() -> string` — Returns `{id}.jsonl`
- `is_uuid() -> boolean` — True when ID is UUID format
- `is_named() -> boolean` — True when ID is human-readable (starts with `-`)

**Commands:** `.show`, `.export`

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 3 | [`.show`](../command/03_show.md) | `session_id::` |
| 6 | [`.export`](../command/06_export.md) | `session_id::` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 13 | [`session::`](../param/13_session.md) | 4 |
| 14 | [`session_id::`](../param/14_session_id.md) | 2 |

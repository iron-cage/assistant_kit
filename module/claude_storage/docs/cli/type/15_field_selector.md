# Type :: 15. `FieldSelector`

### Scope

- **Purpose**: Specify the `FieldSelector` semantic type.
- **Responsibility**: Validation rules, fundamental type, and parameter mapping for `FieldSelector`.
- **In Scope**: Parsing rules, canonical field vocabulary, valid/invalid values, error messages.
- **Out of Scope**: Parameter usage (→ `param/`), command context (→ `command/`), exact rendered layout of projected fields (→ `../command/03_show.md`).

**Purpose:** Attribute-projection selector for `.show`'s entry rendering — names which `Entry`/message attributes to print for each in-scope message, instead of the default chat-log content format. Introduced to satisfy "total control over formatting, projection and filtering" — every attribute the storage layer parses is addressable, individually or in any combination, via this type.

**Fundamental Type:** Wrapper around a comma-separated list of field-name tokens (`Vec<String>` after validation)

**Constants (canonical field vocabulary, 18 names):**

*Entry-level (present on every entry, regardless of role):*
- `uuid`, `parent_uuid`, `timestamp`, `entry_type`, `cwd`, `session_id`, `version`, `git_branch`, `user_type`, `is_sidechain`

*Content (present on every entry; internal shape depends on role — see `../command/03_show.md`'s projection-mode rendering):*
- `content` — for an assistant entry, expands to every `ContentBlock`'s own fields: text block `text`; thinking block `thinking` and `signature`; tool-use block `id`, `name`, and full JSON `input`; tool-result block `tool_use_id`, `content`, and `is_error` (successful results included, not just errors)

*User-only (renders as `—` on an `assistant` entry, or on a `user` entry with no `thinking_metadata` present):*
- `thinking_level`, `thinking_disabled` — from the user message's optional `thinking_metadata` (extended-thinking settings)

*Assistant-only (renders as `—` on a `user` entry):*
- `model`, `message_id`, `stop_reason`, `stop_sequence`, `request_id`

*Special token:*
- `ALL` = `"all"` — expands to the full 18-name vocabulary above, in the canonical order listed

**Constraints:**
- Value is a comma-separated list of tokens: `fields::uuid,timestamp,content`
- Each token, case-insensitive on parse, must match one of the 18 canonical names or the special token `all`
- `all` cannot be combined with other tokens (`fields::all,uuid` is an error — redundant and ambiguous about ordering); use `fields::all` alone
- Duplicate tokens collapse to one occurrence, first-position order preserved (`fields::uuid,uuid` behaves as `fields::uuid`)
- Whitespace around commas is trimmed (`fields:: uuid, timestamp ` parses the same as `fields::uuid,timestamp`)
- Empty value (`fields::` with nothing after) is rejected — but never by this type's own `parse()`. A bare trailing `fields::` is intercepted earlier by the CLI's instruction-syntax parser (unilang), which reports its own message (e.g. `"Expected value for named argument 'fields' but found end of instruction"`) before `FieldSelector::parse` is ever invoked — the same pre-existing behavior every other non-empty-string parameter in this CLI already exhibits (`query::`, `topic::`, `path::`, etc.). This type's own `"fields must be non-empty"` message (below) is reachable only via a direct `FieldSelector::parse("")` call — e.g. unit tests — never through CLI input
- Error on invalid token: `"unknown field '{token}' — valid fields: uuid, parent_uuid, timestamp, entry_type, cwd, session_id, version, git_branch, user_type, is_sidechain, content, thinking_level, thinking_disabled, model, message_id, stop_reason, stop_sequence, request_id, or all"`

**Parsing:**
```
Parse comma-separated string to Vec<FieldName> (case-insensitive per token):
  Input: "uuid,timestamp"    → [Uuid, Timestamp]
  Input: "all"               → [Uuid, ParentUuid, Timestamp, EntryType, Cwd, SessionId,
                                 Version, GitBranch, UserType, IsSidechain, Content,
                                 ThinkingLevel, ThinkingDisabled,
                                 Model, MessageId, StopReason, StopSequence, RequestId]
  Input: "UUID, Timestamp"   → [Uuid, Timestamp]  (case-insensitive, whitespace-trimmed)
  Input: "uuid,uuid"         → [Uuid]              (de-duplicated)
  Input: ""                  → Error: "fields must be non-empty"  (direct parse() call only —
                                 a bare CLI `fields::` never reaches here; see Constraints above)
  Input: "all,uuid"          → Error: "'all' cannot be combined with other fields"
  Input: "bogus"              → Error: "unknown field 'bogus' — valid fields: uuid, parent_uuid, ..."
```

**Methods:**
- `fields() -> Vec<&str>` — Canonical lowercase field names, in request order (or full-vocabulary order when `all` was given)

**Commands:** [`.show`](../command/03_show.md)

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 3 | [`.show`](../command/03_show.md) | `fields::` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 32 | [`fields::`](../param/32_fields.md) | 1 |

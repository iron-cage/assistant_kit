# Parameter :: 32. `fields::`

### Scope

- **Purpose**: Specify the `fields::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `fields::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Attribute-projection selector for `.show`. When given, every in-scope message renders as an explicit field-by-field block (one line per requested attribute) instead of the default chat-log content format — the mechanism behind "total control over formatting, projection and filtering": any single attribute, or any combination, on any message.

**Type:** [`FieldSelector`](../type/15_field_selector.md)

**Fundamental Type:** Comma-separated string list wrapper

**Constraints:**
- Comma-separated list of field-name tokens, case-insensitive, or the single special token `all`
- Each token must be one of the 18 canonical names (see [`FieldSelector`](../type/15_field_selector.md)) or `all`
- `all` cannot be combined with other tokens
- Error on invalid token: `"unknown field '{token}' — valid fields: uuid, parent_uuid, timestamp, entry_type, cwd, session_id, version, git_branch, user_type, is_sidechain, content, thinking_level, thinking_disabled, model, message_id, stop_reason, stop_sequence, request_id, or all"`

**Default:** — (omitted entirely) — output stays the default chat-log content format; `fields::` is purely opt-in and changes no byte of the default rendering

**Commands:** [`.show`](../command/03_show.md)

**Purpose:** Every attribute the storage layer parses off a JSONL entry is addressable, individually or in combination — `fields::timestamp` shows just when each message was sent, `fields::uuid,model` shows two attributes side by side, `fields::all` prints everything the entry carries (including attributes the default content view silently drops: `parent_uuid`, `cwd`, `session_id`, `version`, `git_branch`, `user_type`, `is_sidechain`, user `thinking_level`/`thinking_disabled` extended-thinking settings, assistant `model`/`message_id`/`stop_reason`/`stop_sequence`/`request_id`, tool-call `id`/`name`/full JSON `input`, tool-result `tool_use_id`/`is_error` and successful — not just error — results, and thinking-block `signature`). Applies to every place `.show` renders message content: the session-detail branches (`session_id::` given) and the project-overview tail window (`session_id::` omitted); has no effect in `show_metadata::1` mode's summary block (only the per-entry rendering it may append). Combine with [`index::`](33_index.md) to project fields from exactly one message instead of every in-scope message.

**Examples:**
```bash
# Just the timestamp of every message in a session
.show session_id::abc123 fields::timestamp

# uuid and model side by side
.show session_id::abc123 fields::uuid,model

# Every attribute the entry carries
.show session_id::abc123 fields::all

# One field, from the project-overview tail window
.show fields::content

# One specific message's full attribute set (see index::)
.show session_id::abc123 fields::all index::3
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`FieldSelector`](../type/15_field_selector.md) | Comma-separated list wrapper | String | One of 18 canonical field names, or `all` alone |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 3 | [`.show`](../command/03_show.md) | — (chat-log format) | Applies to session-detail and project-overview tail-window rendering |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |

# Parameter :: 14. `session_id::`

### Scope

- **Purpose**: Specify the `session_id::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `session_id::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Direct session identifier for single-session operations.

**Type:** [`SessionId`](../type/09_session_id.md)

**Fundamental Type:** String (filename stem)

**Constraints:**
- Non-empty string
- Error if session not found: `"session not found: {value}"`

**Default:** none (optional in `.show`, required in `.export`)

**Commands:** `.show`, `.export`

**Purpose:** Identifies a specific session by its filename stem (JSONL filename without `.jsonl`). Required for `.export`; optional for `.show` (omitting it shows the project instead).

**Examples:**
```bash
# Named sessions
session_id::-default_topic
session_id::-commit

# UUID sessions
session_id::8d795a1c-c81d-4010-8d29-b4e678272419
```

**Group:** [Session Identification](../param_group/03_session_identification.md)

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`SessionId`](../type/09_session_id.md) | String (filename stem) | String | Non-empty; session must exist |

### Referenced Parameter Groups
| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 3 | [Session Identification](../param_group/03_session_identification.md) | Full | *(sole member)* |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 3 | [`.show`](../command/03_show.md) | unset — optional | Optional; triggers session search when provided |
| 6 | [`.export`](../command/06_export.md) | none — required | Identifies the session to export |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |
| 3 | [Export Session for Review](../user_story/003_export_session_for_review.md) | developer |

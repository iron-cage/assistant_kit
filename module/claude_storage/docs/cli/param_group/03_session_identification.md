# Parameter Group :: 3. Session Identification

### Scope

- **Purpose**: Specify the Session Identification parameter group.
- **Responsibility**: Member parameters, coherence semantics, and command usage for Session Identification.
- **In Scope**: Group membership, shared behavior, command interactions.
- **Out of Scope**: Individual parameter specs (→ `param/`), type constraints (→ `type/`).

**Parameters:** `session_id::`

**Pattern:** Direct session access by exact identifier

**Purpose:** Identifies a specific session by its filename stem for single-session operations (display or export). When used without an accompanying `project::` parameter, `session_id::` triggers a global search across all projects — the first project containing a matching session is used.

**Used By:** `.show`, `.export` (2 commands total)

**Semantic Coherence Test:**
- "Does `session_id::` identify a specific session for direct access?" → YES

**Why NOT `session::` (filter):**
- `session::` is a substring filter for *narrowing a listing* — it affects which sessions appear in results
- `session_id::` identifies *exactly one* session for a direct operation
- Different semantic purpose: filter expression vs direct identifier

**Parameter Details:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `session_id::` | [`SessionId`](../type/09_session_id.md) | optional in `.show`, required in `.export` | Session filename stem (without `.jsonl`) |

**Examples:**
```bash
.show session_id::-default_topic
.export session_id::-default_topic output::conversation.md
```

### Referenced Commands

| # | Command | Membership | Excluded Params |
|---|---------|------------|-----------------|
| 3 | [`.show`](../command/03_show.md) | Full | — |
| 6 | [`.export`](../command/06_export.md) | Full | — |

### Referenced Parameters

| # | Parameter | Type | Default | Role in Group |
|---|-----------|------|---------|---------------|
| 14 | [`session_id::`](../param/14_session_id.md) | [`SessionId`](../type/09_session_id.md) | unset | Direct session identifier |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |
| 3 | [Export Session for Review](../user_story/003_export_session_for_review.md) | developer |

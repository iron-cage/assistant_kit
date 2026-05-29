# Parameter Group :: 4. Session Filter

**Parameters:** `session::`, `agent::`, `min_entries::`

**Pattern:** Session listing narrowing by session properties

**Purpose:** Together these three parameters filter which sessions appear in a listing — by ID pattern, by session type, and by minimum size.

**Used By (full implementors):** `.list`, `.projects` (2 commands total)

**Partial implementors:**
- `.count` (`session::` only — as exact `SessionId`, not substring filter): scopes entry counting to a session
- `.search` (`session::` only — as exact `SessionId`, not substring filter): restricts search to a session

Note: In `.count` and `.search`, `session::` behaves as a `SessionId` (exact match), not as a `SessionFilter` (substring). The group semantics (substring filtering of session listings) apply only to `.list` and `.projects`.

**Semantic Coherence Test:**
- "Does `session::` control which sessions appear in listing?" → YES (by ID substring) — in `.list` and `.projects`
- "Does `agent::` control which sessions appear in listing?" → YES (by session type)
- "Does `min_entries::` control which sessions appear in listing?" → YES (by size threshold)

**Why NOT `sessions::` (bool):**
- `sessions::` controls whether sessions are shown at all — an on/off toggle for the entire session display tier
- These three parameters determine *which* sessions appear, assuming session display is enabled
- Different semantic level: tier visibility vs session predicate


**Auto-enable behavior:** In `.list`, providing any of `session::`, `agent::`, or `min_entries::` automatically enables `sessions::1`. Override with `sessions::0`.

**Parameter Details:**

| Parameter | Type | Description | Side Effect |
|-----------|------|-------------|-------------|
| `session::` | [`SessionFilter`](../type/08_session_filter.md) | Filter sessions by ID substring | Auto-enables `sessions::1` |
| `agent::` | Boolean | `0`=main only, `1`=agent only, unset=all | Auto-enables `sessions::1` |
| `min_entries::` | [`EntryCount`](../type/01_entry_count.md) | Minimum entry count threshold | Auto-enables `sessions::1` |

**Examples:**
```bash
.list session::commit
.list agent::1
.list agent::0 min_entries::5
.list session::feature agent::0 min_entries::10
```

### Referenced Commands

| # | Command | Membership | Excluded Params | Notes |
|---|---------|------------|-----------------|-------|
| 2 | [`.list`](../command/02_list.md) | Full | — | |
| 4 | [`.count`](../command/04_count.md) | Partial | `agent::`, `min_entries::` | `session::` as SessionId |
| 5 | [`.search`](../command/05_search.md) | Partial | `agent::`, `min_entries::` | `session::` as SessionId |
| 7 | [`.projects`](../command/07_projects.md) | Full | — | |

### Referenced Parameters

| # | Parameter | Type | Default | Role in Group |
|---|-----------|------|---------|---------------|
| 1 | [`agent::`](../param/01_agent.md) | Boolean | unset | Session type filter (main/agent) |
| 7 | [`min_entries::`](../param/07_min_entries.md) | [`EntryCount`](../type/01_entry_count.md) | unset | Minimum entry count threshold |
| 13 | [`session::`](../param/13_session.md) | [`SessionFilter`](../type/08_session_filter.md) | unset | Session ID substring filter |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |

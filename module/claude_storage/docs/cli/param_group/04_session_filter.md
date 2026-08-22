# Parameter Group :: 4. Session Filter

### Scope

- **Purpose**: Specify the Session Filter parameter group.
- **Responsibility**: Member parameters, coherence semantics, and command usage for Session Filter.
- **In Scope**: Group membership, shared behavior, command interactions.
- **Out of Scope**: Individual parameter specs (→ `param/`), type constraints (→ `type/`).

**Parameters:** `session::`, `agent::`, `min_entries::`

**Pattern:** Session listing narrowing by session properties

**Purpose:** Together these three parameters filter which sessions appear in a listing — by ID pattern, by session type, and by minimum size.

**Used By (full implementors):** `.list` (deprecated), `.projects` (2 commands total, 1 deprecated)

**Partial implementors:**
- `.count` (`session::` only — as exact `SessionId`, not substring filter): scopes entry counting to a session
- `.search` (`session::` only — as exact `SessionId`, not substring filter): restricts search to a session

Note: In `.count` and `.search`, `session::` behaves as a `SessionId` (exact match), not as a `SessionFilter` (substring). The group semantics (substring filtering of session listings) apply only to `.list` and `.projects`.

**Semantic Coherence Test:**
- "Does `session::` control which sessions appear in listing?" → YES (by ID substring) — in `.list` and `.projects`
- "Does `agent::` control which sessions appear in listing?" → YES (by session type)
- "Does `min_entries::` control which sessions appear in listing?" → YES (by size threshold)

**Why NOT `show_sessions::` (bool, deprecated) / `detail::` (its successor):**
- `show_sessions::` controlled whether sessions were shown at all — an on/off toggle for the entire session display tier; its successor [`detail::`](../param/30_detail.md) plays the identical tier-visibility role on `.projects`
- These three parameters determine *which* sessions appear, assuming session display is enabled
- Different semantic level: tier visibility vs session predicate — this is also why `detail::` (like `show_sessions::` before it) is not a member of this group either


**Auto-enable behavior (historical — `.list` only, deprecated):** In `.list`, providing any of `session::`, `agent::`, or `min_entries::` automatically enabled `show_sessions::1`, overridable with `show_sessions::0`. `.projects` has no equivalent auto-enable: a session filter narrows the per-project counts shown by the default `detail::projects` view; pass `detail::sessions` explicitly to see individual session lines (see [`../param/30_detail.md`](../param/30_detail.md)).

**Parameter Details:**

| Parameter | Type | Description | Side Effect (current — `.projects`) |
|-----------|------|-------------|-------------|
| `session::` | [`SessionFilter`](../type/08_session_filter.md) | Filter sessions by ID substring | None — narrows already-visible session lines |
| `agent::` | Boolean | `0`=main only, `1`=agent only, unset=all | None — narrows already-visible session lines |
| `min_entries::` | [`EntryCount`](../type/01_entry_count.md) | Minimum entry count threshold | None — narrows already-visible session lines |

**Examples:**
```bash
.projects session::commit
.projects agent::1
.projects agent::0 min_entries::5
.projects session::feature agent::0 min_entries::10
```

### Referenced Commands

| # | Command | Membership | Excluded Params | Notes |
|---|---------|------------|-----------------|-------|
| 2 | [`.list`](../command/02_list.md) (deprecated) | Full | — | Historical; auto-enabled `show_sessions::1` |
| 4 | [`.count`](../command/04_count.md) | Partial | `agent::`, `min_entries::` | `session::` as SessionId |
| 5 | [`.search`](../command/05_search.md) | Partial | `agent::`, `min_entries::` | `session::` as SessionId |
| 7 | [`.projects`](../command/07_projects.md) | Full | — | No auto-enable side effect (see `detail::`) |

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

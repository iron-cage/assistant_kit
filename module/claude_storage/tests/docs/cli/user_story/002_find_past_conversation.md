# User Story :: 2. Find Past Conversation

Acceptance criteria tests for the developer persona locating a past Claude Code conversation.
Source: [002_find_past_conversation.md](../../../../docs/cli/user_story/002_find_past_conversation.md)

## Test Case Index

| ID | Test Name | Criteria |
|----|-----------|---------|
| RWS-1 | List all projects shows projects in storage | AC: list all projects and filter by path |
| RWS-2 | Search by keyword finds matching sessions | AC: search session content by keyword |
| RWS-3 | Project filter restricts search to one project | AC: filter search to a project |
| RWS-4 | Session metadata filters narrow listing | AC: filter by agent type, entry count |
| RWS-5 | Show session displays full session details | AC: display details of a specific session |

---

### RWS-1: List all projects shows projects in storage

**Scenario:** Developer lists all projects to find the relevant one.

**Fixture:** 3 projects with varying path names.

**Command:**
```bash
clg .list
```

**Expected:**
- Stdout lists all 3 project paths, one per line or in a table
- No sessions shown by default (`sessions::0`)

**Exit:** `0`

---

### RWS-2: Search by keyword finds matching sessions

**Scenario:** Developer searches for a keyword they remember from a conversation.

**Fixture:** 2 projects; one session in project A contains the word "authentication"; project B sessions do not.

**Command:**
```bash
clg .search query::authentication
```

**Expected:**
- Stdout includes the matching session from project A
- Stdout includes context showing the matched entry
- Project B sessions are not listed

**Exit:** `0`

---

### RWS-3: Project filter restricts search to one project

**Scenario:** Developer knows which project a conversation is in and narrows the search.

**Fixture:** 2 projects; both have sessions containing "config"; developer knows the target project.

**Command:**
```bash
clg .search query::config project::{target-project-id}
```

**Expected:**
- Only sessions from the specified project appear in results
- Sessions from the other project are excluded

**Exit:** `0`

---

### RWS-4: Session metadata filters narrow listing

**Scenario:** Developer wants only main sessions (not agent sub-sessions) with substantial content.

**Fixture:** Project with 3 sessions: 2 main sessions (1 with 12 entries, 1 with 3 entries) and 1 agent session with 20 entries.

**Command:**
```bash
clg .list sessions::1 agent::0 min_entries::10
```

**Expected:**
- Only the main session with 12 entries appears
- The 3-entry main session is excluded (below threshold)
- The agent session is excluded (`agent::0`)

**Exit:** `0`

---

### RWS-5: Show session displays full session details

**Scenario:** Developer found a session ID and wants to inspect its content.

**Fixture:** One project with a session containing 3 entries with recognizable content.

**Command:**
```bash
clg .show session_id::-default_topic
```

**Expected:**
- Stdout shows session metadata (entry count, timestamps)
- All entries are listed with their type and content snippets

**Exit:** `0`

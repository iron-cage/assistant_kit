# Parameter Group :: 2. Project Scope

**Parameters:** `project::`

**Pattern:** Project-level scope restriction

**Purpose:** Restricts an operation to a specific project, identified by multiple accepted formats.

**Used By:** `.list`, `.show`, `.count`, `.search`, `.export` (5 commands total)

**Semantic Coherence Test:**
- "Does `project::` control which project is operated on?" → YES

**Why NOT `path::` (in `.list`):**
- `path::` in `.list` is a substring filter on project *listing* — it affects which projects are shown, not which single project is the scope
- Different semantic purpose: filter expression vs scope pin

**Why NOT `session_id::`:**
- `session_id::` identifies a session within a project, not the project itself
- Different semantic level: sub-project identifier vs project identifier

**Parameter Details:**

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `project::` | [`ProjectId`](../type/05_project_id.md) | Project identifier (path, encoded ID, UUID, or Path(...) form) | current dir |

**Accepted formats:**
```bash
project::/home/alice/projects/my-app         # Absolute path
project::-home-alice-projects-my-app         # Path-encoded ID
project::8d795a1c-c81d-4010-8d29-b4e678272419  # UUID
project::Path("/home/alice/projects/my-app") # Path(...) from .list output
```

### Referenced Commands

| # | Command | Membership | Excluded Params |
|---|---------|------------|-----------------|
| 2 | [`.list`](../command/02_list.md) | Full | — |
| 3 | [`.show`](../command/03_show.md) | Full | — |
| 4 | [`.count`](../command/04_count.md) | Full | — |
| 5 | [`.search`](../command/05_search.md) | Full | — |
| 6 | [`.export`](../command/06_export.md) | Full | — |

### Referenced Parameters

| # | Parameter | Type | Default | Role in Group |
|---|-----------|------|---------|---------------|
| 10 | [`project::`](../param/10_project.md) | [`ProjectId`](../type/05_project_id.md) | current dir | Project scope pin |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |
| 5 | [Resume Claude Session](../user_story/005_resume_claude_session.md) | developer |

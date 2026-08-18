# Parameter Group :: 2. Project Scope

### Scope

- **Purpose**: Specify the Project Scope parameter group.
- **Responsibility**: Member parameters, coherence semantics, and command usage for Project Scope.
- **In Scope**: Group membership, shared behavior, command interactions.
- **Out of Scope**: Individual parameter specs (→ `param/`), type constraints (→ `type/`).

**Parameters:** `project::`

**Pattern:** Project-level scope restriction

**Purpose:** Restricts an operation to a specific project, identified by multiple accepted formats.

**Used By:** `.list` (deprecated), `.show`, `.count`, `.search`, `.export`, `.projects` (6 commands total, 1 deprecated) — `.projects` uses `project::` to pin the `ids::` scripting path (see [`31_ids.md`](../param/31_ids.md))

**Semantic Coherence Test:**
- "Does `project::` control which project is operated on?" → YES

**Why NOT `path::` (in `.list`, deprecated):**
- `path::` in `.list` was a substring filter on project *listing* — it affected which projects were shown, not which single project was the scope
- That substring-filter role is now `.projects`' [`filter::`](../param/29_filter.md) — still a different semantic purpose from `project::`'s scope pin

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
| 2 | [`.list`](../command/02_list.md) (deprecated) | Full | — |
| 3 | [`.show`](../command/03_show.md) | Full | — |
| 4 | [`.count`](../command/04_count.md) | Full | — |
| 5 | [`.search`](../command/05_search.md) | Full | — |
| 6 | [`.export`](../command/06_export.md) | Full | — |
| 7 | [`.projects`](../command/07_projects.md) | Full | — |

### Referenced Parameters

| # | Parameter | Type | Default | Role in Group |
|---|-----------|------|---------|---------------|
| 10 | [`project::`](../param/10_project.md) | [`ProjectId`](../type/05_project_id.md) | current dir | Project scope pin |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |

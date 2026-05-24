# Find Past Conversation

**Persona:** developer
**Goal:** Locate a specific past Claude Code conversation by project, content, or session metadata.
**Benefit:** Resume or reference earlier work without manually browsing `~/.claude/projects/`.
**Priority:** High

### Acceptance Criteria
- [ ] Can list all projects and filter by path substring
- [ ] Can search session content by keyword
- [ ] Can filter sessions by agent type, minimum entry count, or topic suffix
- [ ] Can display details of a specific session
- [ ] Can view per-project session tree grouped by project

### Referenced Commands
| # | Command | Role |
|---|---------|------|
| 2 | [`.list`](../001_commands.md#command--2-list) | Browse all projects and their sessions |
| 3 | [`.show`](../001_commands.md#command--3-show) | Display full details of a specific session |
| 5 | [`.search`](../001_commands.md#command--5-search) | Full-text search across session content |
| 7 | [`.projects`](../001_commands.md#command--7-projects) | View per-project conversation tree |

### Referenced Parameters
| # | Parameter | Role |
|---|-----------|------|
| 1 | [`agent::`](../004_params.md#parameter--1-agent) | Include or exclude agent sub-sessions |
| 2 | [`case_sensitive::`](../004_params.md#parameter--2-case_sensitive) | Enable case-sensitive keyword matching |
| 3 | [`entries::`](../004_params.md#parameter--3-entries) | Show all session entries in detail view |
| 4 | [`entry_type::`](../004_params.md#parameter--4-entry_type) | Filter search results by entry type |
| 7 | [`min_entries::`](../004_params.md#parameter--7-min_entries) | Filter sessions by minimum entry count |
| 9 | [`path::`](../004_params.md#parameter--9-path) | Restrict to a specific storage root |
| 10 | [`project::`](../004_params.md#parameter--10-project) | Pin search or listing to a specific project |
| 11 | [`query::`](../004_params.md#parameter--11-query) | Keyword to search in session content |
| 12 | [`scope::`](../004_params.md#parameter--12-scope) | Discovery scope for search and listing |
| 13 | [`session::`](../004_params.md#parameter--13-session) | Filter sessions by ID substring |
| 15 | [`sessions::`](../004_params.md#parameter--15-sessions) | Show sessions per project in list view |
| 17 | [`topic::`](../004_params.md#parameter--17-topic) | Filter by session topic suffix |
| 18 | [`type::`](../004_params.md#parameter--18-type) | Filter projects by naming scheme |
| 22 | [`limit::`](../004_params.md#parameter--22-limit) | Cap sessions per project when browsing |

### Referenced Parameter Groups
| # | Parameter Group | Role |
|---|-----------------|------|
| 2 | [Project Scope](../003_parameter_groups.md#project-scope) | Pin lookup to a specific project |
| 3 | [Session Identification](../003_parameter_groups.md#session-identification) | Identify session by ID for `.show` |
| 4 | [Session Filter](../003_parameter_groups.md#session-filter) | Filter sessions by type, agent flag, entry count |
| 5 | [Scope Configuration](../003_parameter_groups.md#scope-configuration) | Narrow discovery scope |

### Related User Stories
| # | User Story | Relationship |
|---|------------|--------------|
| 1 | [Audit Session History](001_audit_session_history.md) | Overlapping commands used for browsing |
| 3 | [Export Session for Review](003_export_session_for_review.md) | Typically export after locating session |
| 5 | [Resume Claude Session](005_resume_claude_session.md) | Typically resume after finding session |

### Workflow Steps

**Step 1: List projects to identify the relevant project**
```bash
cls .list
# Output: all projects; note the project path of interest
```

**Step 2: Search by content keyword**
```bash
cls .search query::authentication
# Output: sessions containing "authentication" with entry excerpts
```

**Step 3: Narrow to a specific project**
```bash
cls .search query::authentication project::my_app
# Output: matching sessions within my_app only
```

**Step 4: Filter by session metadata**
```bash
cls .list sessions::1 path::my_app min_entries::10 agent::0
# Output: main sessions with at least 10 entries in my_app
```

**Step 5: View session details**
```bash
cls .show session_id::abc123
# Output: full session entry list with timestamps and content
```

### Error Handling

**Session not found:**
```bash
cls .search query::keyword scope::all
# If no results: try broader scope or different keyword
```

**Project not recognized:**
```bash
cls .list type::all
# Lists all project types including UUID-named projects
```

### Workflow Variations

**Browse by project tree:**
```bash
cls .projects verbosity::1
# Output: per-project session listing with grouping
```

**Filter agent sub-sessions only:**
```bash
cls .list sessions::1 agent::1
# Output: only agent-spawned sessions
```

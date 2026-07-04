# User Story :: 2. Find Past Conversation

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
| 2 | [`.list`](../command/02_list.md) | Browse all projects and their sessions |
| 3 | [`.show`](../command/03_show.md) | Display full details of a specific session |
| 5 | [`.search`](../command/05_search.md) | Full-text search across session content |
| 7 | [`.projects`](../command/07_projects.md) | View per-project conversation tree |

### Referenced Parameter Groups
| # | Parameter Group | Role |
|---|-----------------|------|
| 2 | [Project Scope](../param_group/02_project_scope.md) | Pin lookup to a specific project |
| 3 | [Session Identification](../param_group/03_session_identification.md) | Identify session by ID for `.show` |
| 4 | [Session Filter](../param_group/04_session_filter.md) | Filter sessions by type, agent flag, entry count |
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | Narrow discovery scope |

### Referenced Parameters
| # | Parameter | Role |
|---|-----------|------|
| 1 | [`agent::`](../param/01_agent.md) | Include or exclude agent sub-sessions |
| 2 | [`case_sensitive::`](../param/02_case_sensitive.md) | Enable case-sensitive keyword matching |
| 3 | [`show_entries::`](../param/03_entries.md) | Show all session entries in detail view |
| 4 | [`entry_type::`](../param/04_entry_type.md) | Filter search results by entry type |
| 7 | [`min_entries::`](../param/07_min_entries.md) | Filter sessions by minimum entry count |
| 9 | [`path::`](../param/09_path.md) | Restrict to a specific storage root |
| 10 | [`project::`](../param/10_project.md) | Pin search or listing to a specific project |
| 11 | [`query::`](../param/11_query.md) | Keyword to search in session content |
| 12 | [`scope::`](../param/12_scope.md) | Discovery scope for search and listing |
| 13 | [`session::`](../param/13_session.md) | Filter sessions by ID substring |
| 14 | [`session_id::`](../param/14_session_id.md) | Identify specific session for detailed view |
| 15 | [`show_sessions::`](../param/15_sessions.md) | Show sessions per project in list view |
| 17 | [`topic::`](../param/17_topic.md) | Filter by session topic suffix |
| 18 | [`type::`](../param/18_type.md) | Filter projects by naming scheme |
| 19 | [`show_stat::`](../param/19_show_stat.md) | Append statistics footer in session view |
| 22 | [`limit::`](../param/22_limit.md) | Cap sessions per project when browsing |
| 24 | [`show_tree::`](../param/24_show_tree.md) | Tree-indent agent sessions in project view |

### Related User Stories
| # | User Story | Relationship |
|---|------------|--------------|
| 1 | [Audit Session History](001_audit_session_history.md) | Overlapping commands used for browsing |
| 3 | [Export Session for Review](003_export_session_for_review.md) | Typically export after locating session |
| 5 | [Resume Claude Session](005_resume_claude_session.md) | Typically resume after finding session |
| 6 | [Quick Context Refresh](006_quick_context_refresh.md) | 006 skips lookup entirely when already in the right directory |

### Workflow Steps

**Step 1: List projects to identify the relevant project**
```bash
cls .list
# /home/user/projects/my_app (14)
# /home/user/projects/web_service (8)
# /home/user/projects/data_pipeline (3)
# c8f7e421-4d3b-4a0e-9fa1-2e5b8c3d7f91 (2)
```

**Step 2: Search by content keyword**
```bash
cls .search query::authentication
# 3 matches
#
# [2024-01-15T14-30-22-abc1] user
#   "The authentication flow needs to handle OAuth tokens properly"
#
# [2024-01-16T09-01-33-def4] assistant
#   "For the authentication service, I recommend using..."
#
# [2024-01-10T11-22-44-jkl0] user
#   "debug the authentication middleware issue"
```

**Step 3: Narrow to a specific project**
```bash
cls .search query::authentication project::my_app
# 2 matches
#
# [2024-01-15T14-30-22-abc1] user
#   "The authentication flow needs to handle OAuth tokens properly"
#
# [2024-01-16T09-01-33-def4] assistant
#   "For the authentication service, I recommend using..."
```

**Step 4: Filter by session metadata**
```bash
cls .list show_sessions::1 path::my_app min_entries::10 agent::0
# /home/user/projects/my_app (4 sessions match filter)
#   2024-01-15T14-30-22-abc1 (24 entries)
#   2024-01-16T09-01-33-def4 (18 entries)
#   2024-01-18T10-05-12-pqr6 (15 entries)
#   2024-01-19T16-44-38-stu9 (11 entries)
```

**Step 5: View session details**
```bash
cls .show session_id::abc123
# === Session: 2024-01-15T14-30-22-abc1 ===
# Project: /home/user/projects/my_app
# Entries: 24
#
# [user] 14:30:22
# The authentication flow needs to handle OAuth tokens properly
#
# [assistant] 14:30:45
# I'll help you implement the OAuth token handling. Let's start by...
#
# [user] 14:31:02
# Can you show me the middleware pattern?
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
cls .projects
# Output: per-project session listing with grouping
```

**Filter agent sub-sessions only:**
```bash
cls .list show_sessions::1 agent::1
# Output: only agent-spawned sessions
```

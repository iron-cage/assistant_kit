# Resume Claude Session

**Persona:** developer
**Goal:** Compute the correct session working directory path and ensure it exists before resuming or starting a Claude Code session.
**Benefit:** Automate the session setup ceremony — checking project history, computing paths, and creating the session directory — with a single reliable sequence of commands.
**Priority:** High

### Acceptance Criteria
- [ ] Can compute the Claude storage path for any local project directory
- [ ] Can verify whether a project has existing conversation history
- [ ] Can compute the session working directory path for a given session and topic
- [ ] Can create the session working directory if it does not exist
- [ ] `.session.ensure` reports the correct resume strategy (resume vs fresh)

### Referenced Commands
| # | Command | Role |
|---|---------|------|
| 8 | [`.project.path`](../001_commands.md#command--8-projectpath) | Compute encoded storage path for a project directory |
| 9 | [`.project.exists`](../001_commands.md#command--9-projectexists) | Verify project has conversation history (exit code check) |
| 10 | [`.session.dir`](../001_commands.md#command--10-sessiondir) | Compute session working directory path |
| 11 | [`.session.ensure`](../001_commands.md#command--11-sessionensure) | Create session directory and report resume strategy |

### Referenced Parameters
| # | Parameter | Role |
|---|-----------|------|
| 9 | [`path::`](../004_params.md#parameter--9-path) | Override default storage root |
| 10 | [`project::`](../004_params.md#parameter--10-project) | Specify project directory for path computation |
| 14 | [`session_id::`](../004_params.md#parameter--14-session_id) | Identify the session to resume |
| 17 | [`topic::`](../004_params.md#parameter--17-topic) | Session topic suffix for workspace organization |
| 20 | [`strategy::`](../004_params.md#parameter--20-strategy) | Override auto-detected resume strategy |

### Referenced Parameter Groups
| # | Parameter Group | Role |
|---|-----------------|------|
| 2 | [Project Scope](../003_parameter_groups.md#project-scope) | Scope operations to a specific project |
| 3 | [Session Identification](../003_parameter_groups.md#session-identification) | Identify session by ID or topic |
| 5 | [Scope Configuration](../003_parameter_groups.md#scope-configuration) | path:: override for non-default storage |

### Related User Stories
| # | User Story | Relationship |
|---|------------|--------------|
| 2 | [Find Past Conversation](002_find_past_conversation.md) | Typically look up session before resuming |

### Workflow Steps

**Step 1: Check if the project has conversation history**
```bash
cls .project.exists project::/home/user/myproject
# Exit 0: project exists; exit 1: no history yet
```

**Step 2: Compute the Claude storage path**
```bash
storage_path=$(cls .project.path project::/home/user/myproject)
echo "Storage path: $storage_path"
```

**Step 3: Compute the session working directory**
```bash
session_dir=$(cls .session.dir session_id::abc123 topic::auth)
echo "Session dir: $session_dir"
```

**Step 4: Ensure session directory exists**
```bash
cls .session.ensure session_id::abc123 topic::auth
# Output: strategy (resume or fresh) and directory path
# Side effect: creates directory on disk if absent
```

### Error Handling

**Project has no history (exit 1 from .project.exists):**
```bash
if ! cls .project.exists project::/home/user/myproject; then
  echo "No history — starting fresh session"
fi
```

**Strategy override when auto-detection is wrong:**
```bash
cls .session.ensure session_id::abc123 topic::auth strategy::fresh
# Forces fresh session even if prior session exists
```

### Workflow Variations

**Full session setup in one script:**
```bash
PROJECT=/home/user/myproject
SESSION_ID=$(cls .list type::conversation project::$PROJECT | head -1)
cls .session.ensure session_id::$SESSION_ID topic::auth
```

**Inspect computed paths without creating anything:**
```bash
cls .project.path project::$PROJECT
cls .session.dir session_id::$SESSION_ID topic::auth
# Both are read-only — only .session.ensure writes to disk
```

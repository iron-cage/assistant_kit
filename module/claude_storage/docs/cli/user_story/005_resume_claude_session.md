# User Story :: 5. Resume Claude Session

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
| 8 | [`.project.path`](../command/08_project_path.md) | Compute encoded storage path for a project directory |
| 9 | [`.project.exists`](../command/09_project_exists.md) | Verify project has conversation history (exit code check) |
| 10 | [`.session.dir`](../command/10_session_dir.md) | Compute session working directory path |
| 11 | [`.session.ensure`](../command/11_session_ensure.md) | Create session directory and report resume strategy |

### Referenced Parameter Groups
| # | Parameter Group | Role |
|---|-----------------|------|
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | path:: specifies project directory for all four commands |

### Referenced Parameters
| # | Parameter | Role |
|---|-----------|------|
| 9 | [`path::`](../param/09_path.md) | Specify project directory for path/existence/session operations |
| 17 | [`topic::`](../param/17_topic.md) | Session topic suffix for workspace organization |
| 20 | [`strategy::`](../param/20_strategy.md) | Override auto-detected resume strategy |

### Related User Stories
| # | User Story | Relationship |
|---|------------|--------------|
| 2 | [Find Past Conversation](002_find_past_conversation.md) | Typically look up session before resuming |
| 6 | [Quick Context Refresh](006_quick_context_refresh.md) | Both resume work in a known directory; 006 shows content instead of setting up paths |

### Workflow Steps

**Step 1: Check if the project has conversation history**
```bash
cls .project.exists path::/home/user/myproject
# sessions exist
```

**Step 2: Compute the Claude storage path**
```bash
storage_path=$(cls .project.path path::/home/user/myproject)
echo "Storage path: $storage_path"
# Storage path: /home/user/.claude/projects/-home-user-myproject/
```

**Step 3: Compute the session working directory**
```bash
session_dir=$(cls .session.dir path::/home/user/myproject topic::auth)
echo "Session dir: $session_dir"
# Session dir: /home/user/.claude/projects/-home-user-myproject/-auth/
```

**Step 4: Ensure session directory exists**
```bash
cls .session.ensure path::/home/user/myproject topic::auth
# /home/user/.claude/projects/-home-user-myproject/-auth/
# resume
```

### Error Handling

**Project has no history (exit 1 from .project.exists):**
```bash
if ! cls .project.exists path::/home/user/myproject; then
  echo "No history — starting fresh session"
fi
```

**Strategy override when auto-detection is wrong:**
```bash
cls .session.ensure path::/home/user/myproject topic::auth strategy::fresh
# Forces fresh session even if prior session exists
```

### Workflow Variations

**Full session setup in one script:**
```bash
PROJECT=/home/user/myproject
SESSION_ID=$(cls .list type::conversation project::$PROJECT | head -1)
cls .session.ensure path::$PROJECT topic::auth
```

**Inspect computed paths without creating anything:**
```bash
cls .project.path path::$PROJECT
cls .session.dir path::$PROJECT topic::auth
# Both are read-only — only .session.ensure writes to disk
```

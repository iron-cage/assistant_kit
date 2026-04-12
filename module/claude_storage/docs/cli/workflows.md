# Workflows

Common usage patterns for the `claude_storage` CLI. Each workflow shows a practical scenario with commands, expected output shape, and notes.

See [commands.md](commands.md) for command reference and [params.md](params.md) for parameter details.

## Workflow Complexity Matrix

| Workflow | Commands Used | Complexity |
|----------|--------------|------------|
| [Quick storage check](#1-quick-storage-check) | `.status` | low |
| [Find a past conversation](#2-find-a-past-conversation) | `.list`, `.show` | low |
| [Export a conversation](#3-export-a-conversation) | `.show`, `.export` | low |
| [Search for a topic](#4-search-for-a-topic) | `.search`, `.show` | low |
| [Count storage by scope](#5-count-storage-by-scope) | `.count` | low |
| [Inspect agent sessions](#6-inspect-agent-sessions) | `.list`, `.show` | medium |
| [Navigate project hierarchy](#7-navigate-project-hierarchy) | `.session`, `.list`, `.show` | medium |
| [Find substantial sessions](#8-find-substantial-sessions) | `.list` | medium |
| [Session lifecycle management](#9-session-lifecycle-management) | `.path`, `.exists`, `.session.dir`, `.session.ensure` | medium |

---

## Common Workflows

### 1. Quick storage check

**Scenario:** You want to see how much conversation history exists and where the storage root is.

```bash
# Default summary
claude_storage .status
# Output: projects: N, sessions: N, storage root path

# Detailed per-project breakdown
claude_storage .status verbosity::2
# Output: summary table + per-project session counts and entry counts
```

**Notes:** Use `verbosity::0` for scripting: `count=$(claude_storage .status v::0 | grep projects | cut -d: -f2)`.

---

### 2. Find a past conversation

**Scenario:** You remember working on a specific project last week and want to find the session.

```bash
# Step 1: List projects matching path keyword
claude_storage .list path::claude_tools
# Output: projects whose paths contain "claude_tools"

# Step 2: List sessions for a matching project
claude_storage .list path::claude_tools sessions::1
# Output: same projects, now with their sessions listed

# Step 3: Show the session content
claude_storage .show project::/home/user1/pro/lib/consumer session_id::-default_topic
# Output: conversation content
```

**Notes:** If the project matches the current directory, `.show` without `project::` uses the cwd automatically.

---

### 3. Export a conversation

**Scenario:** You want to save a conversation as a Markdown file to share or review offline.

```bash
# Step 1: Find the session ID (if unknown)
claude_storage .show
# Output: lists sessions for current project; note the session ID

# Step 2: Export it
claude_storage .export session_id::-default_topic output::conversation.md
# Output: writes conversation.md in current directory

# Alternative: export as JSON for programmatic processing
claude_storage .export session_id::-default_topic format::json output::session.json
```

**Notes:** `output::` parent directory must exist; the file is overwritten without warning.

---

### 4. Search for a topic

**Scenario:** You remember discussing a specific feature ("session management") but don't know which project or session.

```bash
# Step 1: Broad search across all storage
claude_storage .search query::"session management"
# Output: list of matching sessions with snippets

# Step 2: Narrow to user messages only (what you asked about)
claude_storage .search query::"session management" entry_type::user
# Output: sessions where you asked about session management

# Step 3: Show the full conversation
claude_storage .show session_id::FOUND_SESSION_ID project::FOUND_PROJECT
```

**Notes:** Without `project::`, search scans all projects — may be slow on large storage (2000+ projects). Narrow with `project::` when project is known.

---

### 5. Count storage by scope

**Scenario:** You want numbers without loading listings.

```bash
# Count all projects
claude_storage .count
# Output: projects: N

# Count sessions in a project
claude_storage .count target::sessions project::-home-user1-pro-lib-consumer
# Output: sessions: N

# Count entries in a session
claude_storage .count target::entries project::-home-user1-pro-lib-consumer session::-default_topic
# Output: entries: N
```

**Notes:** `.count` is optimized for performance and avoids loading full session content.

---

### 6. Inspect agent sessions

**Scenario:** You want to understand what sub-agents were spawned during a Claude Code session.

```bash
# Step 1: Find agent sessions for the current project
claude_storage .list sessions::1 agent::1
# Output: projects with agent-only sessions listed

# Step 2: Show a specific agent session
claude_storage .show session_id::agent-abc123
# Output: agent session conversation content

# Step 3: Show agent sessions with metadata only
claude_storage .show session_id::agent-abc123 metadata::1
# Output: technical metadata (entry count, timestamps, agentId) without content
```

**Notes:** Agent sessions are stored as `agent-*.jsonl` files. They have `isSidechain: true` and carry an `agentId` linking them to the parent session.

---

### 7. Navigate project hierarchy

**Scenario:** You're in a subdirectory and want to find all sessions in ancestor projects.

```bash
# Step 1: Check if current directory has history
claude_storage .session
# Exit 0: has history; Exit 1: no history

# Step 2: Check a parent directory
claude_storage .session path::/home/user1/pro
# Exit 0: parent has history too

# Step 3: List all sessions for current project
claude_storage .list sessions::1 path::current_dir_name
# Output: sessions for matching projects

# Available via .projects:
# claude_storage .projects scope::relevant
# Output: sessions from all ancestor projects in one listing
```

**Notes:** When `.projects` with `scope::relevant` is implemented, this workflow will be replaced by a single command.

---

### 8. Find substantial sessions

**Scenario:** You want to find substantive conversations (skip short one-message sessions).

```bash
# Find sessions with at least 20 entries
claude_storage .list min_entries::20
# Output: projects with sessions meeting the threshold, sessions auto-shown

# Find substantive agent sessions
claude_storage .list agent::1 min_entries::10
# Output: agent sessions with 10+ entries

# Find substantial sessions in a specific project by path
claude_storage .list path::claude_tools min_entries::50
# Output: claude_tools projects with sessions having 50+ entries
```

**Notes:** `min_entries::` counts all entries including both user and assistant turns, so a 10-entry session has roughly 5 user-assistant exchange pairs.

---

### 9. Session lifecycle management

**Scenario:** A shell script needs to set up a session working directory, detect whether it's a resume or a fresh start, and report the path to the caller.

```bash
# Step 1: Inspect the storage path for the project
claude_storage .path path::/home/user/project topic::work
# Output: /home/user/.claude/projects/-home-user-project--work/

# Step 2: Check if history exists before doing anything
if clg .exists path::/home/user/project topic::work; then
  echo "Will resume existing conversation"
else
  echo "Will start fresh"
fi

# Step 3: Compute session directory without creating it
SESSION_DIR=$(clg .session.dir path::/home/user/project topic::work)
# Output: /home/user/project/-work

# Step 4: Ensure directory exists and get strategy in one call
result=$(clg .session.ensure path::/home/user/project topic::work)
session_dir=$(echo "$result" | head -1)
strategy=$(echo "$result" | tail -1)
echo "Session directory: $session_dir"
echo "Strategy: $strategy"     # "resume" or "fresh"

# Step 5: Force a specific strategy
result=$(clg .session.ensure path::/home/user/project topic::work strategy::fresh)
# Output (two lines):
# /home/user/project/-work
# fresh
```

**Notes:**
- `.path` and `.session.dir` never modify the filesystem; they only compute paths
- `.exists` is equivalent to `.session` for detection but is documented to exit 1 intentionally for scripting
- `.session.ensure` is the only command that creates directories; it is idempotent (safe to call repeatedly)
- All four commands accept `.`, `..`, `~`, and `~/path` in `path::`

---

## Best Practices

**Narrow scope before broad search:** Use `project::` or `path::` to restrict expensive operations. Without scoping, `.search` reads every session in storage.

**Use verbosity levels for scripting:** `v::0` produces minimal output suitable for piping; `v::1` is for human review; `v::2` or `v::3` for debugging.

**Prefer `.count` over `.list` for numbers:** `.count` avoids loading session content and is much faster on large storage.

**Use `sessions::0` with session filters for project discovery:** `claude_storage .list session::commit sessions::0` finds which projects have sessions matching "commit" without expanding the session list.

**Session IDs from `.show` output:** When you see sessions listed by `.show` (project view), copy the session ID directly into `session_id::` for the next command.

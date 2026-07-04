# User Story :: 6. Quick Context Refresh

**Persona:** developer
**Goal:** See the most recent conversation content in the current directory immediately, without specifying a project, session ID, or search query.
**Benefit:** Resume work instantly after stepping away, without remembering session IDs or running multi-step lookup commands.
**Priority:** Medium

### Acceptance Criteria
- [ ] Can view the last few conversation entries for the current directory with zero parameters
- [ ] Can control how many recent entries are shown
- [ ] Can view recent entries for a non-default session topic
- [ ] Reports a clear error when the current directory has no conversation history

### Referenced Commands
| # | Command | Role |
|---|---------|------|
| 12 | [`.tail`](../command/12_tail.md) | Print the last N entries of the current directory's conversation |

### Referenced Parameters
| # | Parameter | Role |
|---|-----------|------|
| 9 | [`path::`](../param/09_path.md) | Directory to resolve the project from (default cwd) |
| 17 | [`topic::`](../param/17_topic.md) | Session topic suffix (default `default_topic`) |
| 25 | [`tail::`](../param/25_tail.md) | Number of trailing entries to print (default 4) |

### Related User Stories
| # | User Story | Relationship |
|---|------------|--------------|
| 2 | [Find Past Conversation](002_find_past_conversation.md) | 002 searches when the location is unknown; 006 skips lookup when already in the right directory |
| 5 | [Resume Claude Session](005_resume_claude_session.md) | Both resume work in a known directory; 006 shows content instead of setting up paths |

### Workflow Steps

**Step 1: Peek at the last few messages without any arguments**
```bash
cd /home/user/myproject
cls .tail
# Prints the last 4 entries of the default-topic session for this directory
```

**Step 2: Show more (or fewer) entries**
```bash
cls .tail tail::10
# Prints the last 10 entries
```

**Step 3: Check a non-default topic**
```bash
cls .tail topic::work tail::4
# Prints the last 4 entries of the "work" topic session
```

### Error Handling

**No conversation history for this directory:**
```bash
cls .tail
# Exit 2: "no history found for this project"
```

**Negative tail count:**
```bash
cls .tail tail::-1
# Exit 1: "tail must be non-negative"
```

### Workflow Variations

**Show all entries instead of a fixed count:**
```bash
cls .tail tail::0
# 0 means show all entries, oldest-first
```

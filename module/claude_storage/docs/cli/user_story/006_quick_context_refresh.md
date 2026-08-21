# User Story :: 6. Quick Context Refresh

**Persona:** developer
**Goal:** See the most recent conversation content in the current directory immediately, without specifying a project, session ID, or search query.
**Benefit:** Resume work instantly after stepping away, without remembering session IDs or running multi-step lookup commands.
**Priority:** Medium

### Acceptance Criteria
- [ ] Can view the last few conversation turns for the current directory with zero parameters
- [ ] Can control how many recent turns are shown
- [ ] Can view recent turns for a non-default session topic
- [ ] Can read one long turn in full instead of its folded first lines
- [ ] Can scan a wide span of history at one line per turn
- [ ] Reports a clear error when the current directory has no conversation history

### Referenced Commands
| # | Command | Role |
|---|---------|------|
| 12 | [`.tail`](../command/12_tail.md) | Print the last N turns of the current directory's conversation |

### Referenced Parameters
| # | Parameter | Role |
|---|-----------|------|
| 9 | [`path::`](../param/09_path.md) | Directory to resolve the project from (default cwd) |
| 17 | [`topic::`](../param/17_topic.md) | Session topic suffix (default: unset — falls back to the most recently modified session) |
| 25 | [`last::`](../param/25_last.md) | Number of trailing turns to print (default 4) |
| 42 | [`full::`](../param/42_full.md) | Print every body line instead of folding turns past 8 lines |
| 43 | [`compact::`](../param/43_compact.md) | One line per turn instead of full bodies |

**Unit note:** `.tail` counts *turns*, not raw JSONL entries — one assistant response spans several records sharing a `message.id` and renders as a single turn. `.tail last::4` and `.show last::4` therefore select different amounts of history; see [`command/12_tail.md`](../command/12_tail.md) § Turn Grouping.

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
# Prints the last 4 turns of the most recently modified session for this directory
```

**Step 2: Show more (or fewer) turns**
```bash
cls .tail last::10
# Prints the last 10 turns
```

**Step 3: Check a non-default topic**
```bash
cls .tail topic::work last::4
# Prints the last 4 turns of the "work" topic session
```

### Error Handling

**No conversation history for this directory:**
```bash
cls .tail
# Exit 2: "no history found for this project"
```

**Negative tail count:**
```bash
cls .tail last::-1
# Exit 1: "last must be non-negative"
```

### Workflow Variations

**Show all turns instead of a fixed count:**
```bash
cls .tail last::0
# 0 means show all turns, oldest-first
```

**Read one long turn in its entirety:**
```bash
cls .tail last::1 full::1
# Lifts the default 8-line fold, so nothing is replaced by a "⋯ N more lines" hint
```

**Scan a wide span at a glance:**
```bash
cls .tail compact::1 last::40
# One row per turn — ordinal, age, speaker, and a truncated first line
```

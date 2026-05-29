# Audit Session History

**Persona:** developer
**Goal:** Get an at-a-glance overview of all Claude Code projects and session counts in local storage.
**Benefit:** Quickly understand storage usage and verify the storage root is working correctly without manually browsing the filesystem.
**Priority:** High

### Acceptance Criteria
- [ ] Can view total project count and session count in one command
- [ ] Can drill into per-project session counts
- [ ] Can count specific targets (projects, sessions, entries) independently
- [ ] Can override the storage root to inspect an alternate location
- [ ] Can view token usage breakdown with show_tokens::1

### Referenced Commands
| # | Command | Role |
|---|---------|------|
| 1 | [`.status`](../command/01_status.md) | Primary overview: project totals, session totals, storage root |
| 2 | [`.list`](../command/02_list.md) | Enumerate projects with per-project session expansion |
| 4 | [`.count`](../command/04_count.md) | Precise item counts for specific targets |

### Referenced Parameters
| # | Parameter | Role |
|---|-----------|------|
| 9 | [`path::`](../param/09_path.md) | Override default storage root for inspection |
| 15 | [`sessions::`](../param/15_sessions.md) | Expand session list per project in `.list` |
| 16 | [`target::`](../param/16_target.md) | Specify count target (projects, sessions, entries) |
| 23 | [`show_tokens::`](../param/23_show_tokens.md) | Show token usage section in .status |

### Referenced Parameter Groups
| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Output Control](../param_group/01_output_control.md) | show_tokens:: enables token usage in .status |
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | path:: override to inspect alternate storage |

### Related User Stories
| # | User Story | Relationship |
|---|------------|--------------|
| 4 | [Query Storage Programmatically](004_query_storage_programmatically.md) | Same commands used in scripting/automation context |

### Workflow Steps

**Step 1: Check overall storage summary**
```bash
cls .status
# Output: summary table with total projects and sessions
```

**Step 2: View token usage**
```bash
cls .status show_tokens::1
# Output: adds entry counts and token breakdown (input, output, cache)
```

**Step 3: Count sessions precisely**
```bash
cls .count target::sessions
# Output: exact session count as a number
```

**Step 4: List all projects with sessions**
```bash
cls .list sessions::1
# Output: all projects, each with their sessions listed below
```

### Workflow Variations

**Inspect alternate storage location:**
```bash
cls .status path::/backup/.claude
cls .count target::projects path::/backup/.claude
```

**Full token audit:**
```bash
cls .status show_tokens::1
# Output: includes entry counts and token totals (slow — parses all JSONL)
```

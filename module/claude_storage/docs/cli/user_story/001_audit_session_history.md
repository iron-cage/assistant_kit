# User Story :: 1. Audit Session History

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
- [ ] Can view a per-session usage table (turns, tokens, cache, duration, directory) across a scoped set of sessions — see [`.usage`](../command/13_usage.md)
- [ ] Can view a grouped, sorted, column-projected token-usage table — by session, project, model, or day — to compare cost across dimensions rather than reading one row per session — see [`.table`](../command/14_table.md)

### Referenced Commands
| # | Command | Role |
|---|---------|------|
| 1 | [`.status`](../command/01_status.md) | Primary overview: project totals, session totals, storage root |
| 7 | [`.projects`](../command/07_projects.md) | Enumerate projects with per-project session expansion |
| 4 | [`.count`](../command/04_count.md) | Precise item counts for specific targets |
| 13 | [`.usage`](../command/13_usage.md) | Per-session usage table for detailed activity audit |
| 14 | [`.table`](../command/14_table.md) | Grouped/sorted/projected token-usage table for cross-sectional cost comparison |

### Referenced Parameter Groups
| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Output Control](../param_group/01_output_control.md) | show_tokens:: enables token usage in .status |
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | path:: override to inspect alternate storage |

### Referenced Parameters
| # | Parameter | Role |
|---|-----------|------|
| 9 | [`path::`](../param/09_path.md) | Override default storage root for inspection |
| 30 | [`detail::`](../param/30_detail.md) | Select terse (`projects`) or full (`sessions`, default) view in `.projects` |
| 16 | [`target::`](../param/16_target.md) | Specify count target (projects, sessions, entries) |
| 23 | [`show_tokens::`](../param/23_show_tokens.md) | Show token usage section in .status |

### Related User Stories
| # | User Story | Relationship |
|---|------------|--------------|
| 4 | [Query Storage Programmatically](004_query_storage_programmatically.md) | Same commands used in scripting/automation context |

### Workflow Steps

**Step 1: Check overall storage summary**
```bash
cls .status
# Storage: ~/.claude/
# Projects: 42 (UUID: 10, Path: 32)
# Sessions: 187 (Main: 164, Agent: 23)
```

**Step 2: View token usage**
```bash
cls .status show_tokens::1
# Storage: ~/.claude/
# Projects: 42 (UUID: 10, Path: 32)
# Sessions: 187 (Main: 164, Agent: 23)
# Entries: 4293 (User: 2147, Assistant: 2146)
# Tokens:
#   Input:          1423891
#   Output:          312847
#   Cache Read:      891234
#   Cache Creation:  623481
```

**Step 3: Count sessions precisely**
```bash
cls .count target::sessions
# 187
```

**Step 4: List all projects with sessions**
```bash
cls .projects
# /home/user/projects/my_app
#   2024-01-15T14-30-22-abc1 (24 entries)
#   2024-01-16T09-01-33-def4 (18 entries)
#   2024-01-17T11-22-44-ghi7 (6 entries)
# /home/user/projects/web_service
#   2024-01-10T11-22-44-jkl0 (31 entries)
#   2024-01-12T08-15-00-mno3 (12 entries)
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

**Per-session usage audit (see [`.usage`](../command/13_usage.md)):**
```bash
cls .usage scope::under path::/data/repos/yrd_review
# Session   Command   Turns      In     Out   Cache      Dur  Dir
# bf61b676  /role        31   44.8k  105.8k   4.8M    5m24s  /data/repos/yrd_review/2026_troy_venue_pipeline_dev/pr_101
```

**Cost comparison across projects (see [`.table`](../command/14_table.md)):**
```bash
cls .table scope::global group::project sort::total limit::10
# Group                     Sessions   Calls     Input    Output     Cache    MaxCtx     Total     Pct
# /data/repos/yrd_review           1       4       500       300       200       700      1.0k   83.3%
# /data/repos/other_app            1       2       100        50        50       150       200   16.7%
```

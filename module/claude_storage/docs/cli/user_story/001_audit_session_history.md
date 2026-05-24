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
- [ ] Output at verbosity::0 is suitable for use in scripts

### Referenced Commands
| # | Command | Role |
|---|---------|------|
| 1 | [`.status`](../001_commands.md#command--1-status) | Primary overview: project totals, session totals, storage root |
| 2 | [`.list`](../001_commands.md#command--2-list) | Enumerate projects with per-project session expansion |
| 4 | [`.count`](../001_commands.md#command--4-count) | Precise item counts for specific targets |

### Referenced Parameters
| # | Parameter | Role |
|---|-----------|------|
| 9 | [`path::`](../004_params.md#parameter--9-path) | Override default storage root for inspection |
| 15 | [`sessions::`](../004_params.md#parameter--15-sessions) | Expand session list per project in `.list` |
| 16 | [`target::`](../004_params.md#parameter--16-target) | Specify count target (projects, sessions, entries) |
| 19 | [`verbosity::`](../004_params.md#parameter--19-verbosity) | Control output detail level |

### Referenced Parameter Groups
| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Output Control](../003_parameter_groups.md#output-control) | Controls verbosity across status, list, and count |
| 5 | [Scope Configuration](../003_parameter_groups.md#scope-configuration) | path:: override to inspect alternate storage |

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

**Step 2: View per-project breakdown**
```bash
cls .status verbosity::2
# Output: per-project session counts and entry breakdowns
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

**Machine-readable output for scripts:**
```bash
cls .status verbosity::0
# Output: "projects: N, sessions: N" — suitable for piping
```

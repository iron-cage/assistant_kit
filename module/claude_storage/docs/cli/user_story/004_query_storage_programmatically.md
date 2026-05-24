# Query Storage Programmatically

**Persona:** developer
**Goal:** Query Claude Code storage from a script to get machine-readable output for integration with other tools.
**Benefit:** Embed storage statistics and session data in dashboards, monitoring scripts, or data pipelines without parsing human-readable terminal output.
**Priority:** Medium

### Acceptance Criteria
- [ ] `.status verbosity::0` outputs key=value pairs suitable for parsing
- [ ] `.count` outputs a bare integer with no decorations
- [ ] Can query count for a specific target (projects, sessions, entries)
- [ ] Can scope queries to a specific storage root via `path::` or `CLAUDE_STORAGE_ROOT`
- [ ] All commands exit 0 on success and non-zero on error

### Referenced Commands
| # | Command | Role |
|---|---------|------|
| 1 | [`.status`](../001_commands.md#command--1-status) | Machine-readable storage summary at verbosity::0 |
| 2 | [`.list`](../001_commands.md#command--2-list) | Enumerate projects for scripted iteration |
| 4 | [`.count`](../001_commands.md#command--4-count) | Bare integer count output for comparison and thresholds |

### Referenced Parameters
| # | Parameter | Role |
|---|-----------|------|
| 9 | [`path::`](../004_params.md#parameter--9-path) | Point to a specific storage root |
| 12 | [`scope::`](../004_params.md#parameter--12-scope) | Constrain discovery to a project or subtree |
| 16 | [`target::`](../004_params.md#parameter--16-target) | Specify what to count (projects, sessions, entries) |
| 19 | [`verbosity::`](../004_params.md#parameter--19-verbosity) | Set to 0 for machine-readable output |
| 21 | [`count::`](../004_params.md#parameter--21-count) | Output count only as bare integer for scripting |

### Referenced Parameter Groups
| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Output Control](../003_parameter_groups.md#output-control) | verbosity::0 enables machine-readable mode |
| 5 | [Scope Configuration](../003_parameter_groups.md#scope-configuration) | path:: and scope:: for integration targets |

### Referenced Formats
| # | Format | Role |
|---|--------|------|
| 2 | [json](../format/02_json.md) | Machine-parseable session export for pipeline integration |

### Related User Stories
| # | User Story | Relationship |
|---|------------|--------------|
| 1 | [Audit Session History](001_audit_session_history.md) | Same commands used in interactive context |

### Workflow Steps

**Step 1: Get machine-readable storage summary**
```bash
cls .status verbosity::0
# Output: "projects: N, sessions: N"
```

**Step 2: Count sessions for a threshold check**
```bash
session_count=$(cls .count target::sessions)
if [ "$session_count" -gt 100 ]; then
  echo "Warning: large session count"
fi
```

**Step 3: Iterate over projects in a script**
```bash
cls .list verbosity::0 | while read project; do
  echo "Processing: $project"
done
```

### Error Handling

**Non-zero exit on error:**
```bash
if ! cls .status path::/custom; then
  echo "Storage read failed (exit $?)"
  exit 1
fi
```

### Workflow Variations

**Use environment variable instead of path:: flag:**
```bash
export CLAUDE_STORAGE_ROOT=/custom/.claude
cls .status verbosity::0
cls .count target::sessions
```

**Export sessions as JSON for pipeline:**
```bash
cls .export session_id::abc123 format::json output::/tmp/session.json
jq '.messages | length' /tmp/session.json
```

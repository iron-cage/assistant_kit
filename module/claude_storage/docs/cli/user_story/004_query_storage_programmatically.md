# User Story :: 4. Query Storage Programmatically

**Persona:** developer
**Goal:** Query Claude Code storage from a script to get machine-readable output for integration with other tools.
**Benefit:** Embed storage statistics and session data in dashboards, monitoring scripts, or data pipelines without parsing human-readable terminal output.
**Priority:** Medium

### Acceptance Criteria
- [ ] `.status` outputs parseable plain-text storage summary
- [ ] `.count` outputs a bare integer with no decorations
- [ ] Can query count for a specific target (projects, sessions, entries)
- [ ] Can scope queries to a specific storage root via `path::` or `CLAUDE_STORAGE_ROOT`
- [ ] All commands exit 0 on success and non-zero on error

### Referenced Commands
| # | Command | Role |
|---|---------|------|
| 1 | [`.status`](../command/01_status.md) | Plain-text storage summary (parseable) |
| 2 | [`.list`](../command/02_list.md) | Enumerate projects for scripted iteration |
| 4 | [`.count`](../command/04_count.md) | Bare integer count output for comparison and thresholds |

### Referenced Formats
| # | Format | Role |
|---|--------|------|
| 2 | [json](../format/02_json.md) | Machine-parseable session export for pipeline integration |

### Referenced Parameter Groups
| # | Parameter Group | Role |
|---|-----------------|------|
| 5 | [Scope Configuration](../param_group/05_scope_configuration.md) | path:: and scope:: for integration targets |

### Referenced Parameters
| # | Parameter | Role |
|---|-----------|------|
| 9 | [`path::`](../param/09_path.md) | Point to a specific storage root |
| 12 | [`scope::`](../param/12_scope.md) | Constrain discovery to a project or subtree |
| 16 | [`target::`](../param/16_target.md) | Specify what to count (projects, sessions, entries) |
| 21 | [`count::`](../param/21_count.md) | Output count only as bare integer for scripting |

### Related User Stories
| # | User Story | Relationship |
|---|------------|--------------|
| 1 | [Audit Session History](001_audit_session_history.md) | Same commands used in interactive context |

### Workflow Steps

**Step 1: Get storage summary**
```bash
cls .status
# Storage: ~/.claude/
# Projects: 42 (UUID: 10, Path: 32)
# Sessions: 187 (Main: 164, Agent: 23)
```

**Step 2: Count sessions for a threshold check**
```bash
session_count=$(cls .count target::sessions)
if [ "$session_count" -gt 100 ]; then
  echo "Warning: large session count"
fi
# Warning: large session count
```

**Note:** `.count` is optimized for performance and avoids loading full session content — prefer it over `.list` when a script only needs a number.

**Step 3: Iterate over projects in a script**
```bash
cls .list | grep '^/' | while read -r project_line; do
  echo "Processing: $project_line"
done
# Processing: /home/user/projects/my_app (14)
# Processing: /home/user/projects/web_service (8)
# Processing: /home/user/projects/data_pipeline (3)
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
cls .status
cls .count target::sessions
```

**Export sessions as JSON for pipeline:**
```bash
cls .export session_id::abc123 format::json output::/tmp/session.json
jq '.messages | length' /tmp/session.json
```

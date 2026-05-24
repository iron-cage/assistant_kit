# Export Session for Review

**Persona:** developer
**Goal:** Export a Claude Code session transcript to a file in a chosen format for offline review or sharing.
**Benefit:** Preserve conversation history in a portable format (markdown, JSON, or plain text) without depending on Claude Code storage access.
**Priority:** Medium

### Acceptance Criteria
- [ ] Can export a specific session by ID to a named output file
- [ ] Can choose between markdown, JSON, and plain text formats
- [ ] Can export metadata only (without full entry content)
- [ ] Can target a non-default storage root for export
- [ ] Output file is atomically written (no partial files on failure)

### Referenced Commands
| # | Command | Role |
|---|---------|------|
| 6 | [`.export`](../001_commands.md#command--6-export) | Write session transcript to a file in chosen format |

### Referenced Parameters
| # | Parameter | Role |
|---|-----------|------|
| 5 | [`format::`](../004_params.md#parameter--5-format) | Select export rendering mode (markdown, json, text) |
| 6 | [`metadata::`](../004_params.md#parameter--6-metadata) | Export session metadata only, without entry content |
| 8 | [`output::`](../004_params.md#parameter--8-output) | Output file path for the exported file |
| 9 | [`path::`](../004_params.md#parameter--9-path) | Override default storage root |
| 14 | [`session_id::`](../004_params.md#parameter--14-session_id) | Identify the exact session to export |

### Referenced Parameter Groups
| # | Parameter Group | Role |
|---|-----------------|------|
| 3 | [Session Identification](../003_parameter_groups.md#session-identification) | Pin the export to a specific session by ID |
| 5 | [Scope Configuration](../003_parameter_groups.md#scope-configuration) | path:: override for non-default storage |

### Referenced Formats
| # | Format | Role |
|---|--------|------|
| 1 | [markdown](../format/01_markdown.md) | Human-readable export for notes, sharing, and reading |
| 2 | [json](../format/02_json.md) | Machine-parseable export for processing and integration |
| 3 | [text](../format/03_text.md) | Plain text export for piping and minimal readers |

### Related User Stories
| # | User Story | Relationship |
|---|------------|--------------|
| 2 | [Find Past Conversation](002_find_past_conversation.md) | Find the target session before exporting |

### Workflow Steps

**Step 1: Identify the session to export**
```bash
cls .search query::authentication
# Note the session_id from the output
```

**Step 2: Export as markdown (default)**
```bash
cls .export session_id::abc123 output::session.md
# Output: session.md written with all entries in markdown format
```

**Step 3: Export as JSON for processing**
```bash
cls .export session_id::abc123 format::json output::session.json
# Output: session.json with one JSONL entry per line
```

### Error Handling

**Session ID not found:**
```bash
# Exit code 2 — verify session_id with .show first
cls .show session_id::abc123
```

**Output path not writable:**
```bash
# Exit code 1 — check directory permissions or use absolute path
cls .export session_id::abc123 output::/tmp/session.md
```

### Workflow Variations

**Export metadata only (no entry content):**
```bash
cls .export session_id::abc123 metadata::1 output::meta.md
```

**Export plain text for piping:**
```bash
cls .export session_id::abc123 format::text output::session.txt
cat session.txt | grep -i "error"
```

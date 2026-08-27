# Envelope Doc Entity

### Scope

- **Purpose**: Enumerate every top-level line kind Claude Code writes to a session JSONL file, not only the conversation entries.
- **Responsibility**: Master file for the `envelope` collection — one instance per `type` discriminator value, with its payload fields, structural class, and version lifecycle.
- **In Scope**: All 19 observed `type` values; per-kind payload field tables with types and presence rates; observed frequency and version lifecycle.
- **Out of Scope**: The nested dispatch levels — `attachment.type` (→ [`../attachment/`](../attachment/readme.md)) and `system.subtype` (→ [`../system_event/`](../system_event/readme.md)); the common-field contract per class (→ [`../envelope_class/`](../envelope_class/readme.md)); conversation-entry field detail (→ [`../jsonl/`](../jsonl/readme.md)).

**File location**: `~/.claude/projects/{project-id}/{session-id}.jsonl`

**Boundary with [`../jsonl/`](../jsonl/readme.md)**: that collection specifies the *conversation* entry format — the internal shape of `user`/`assistant` lines, their content blocks, usage object, and threading. This collection specifies the *envelope taxonomy* — what kinds of line exist at all, of which `user`/`assistant` are two of 19. A consumer that reads only `jsonl/` will correctly parse conversation entries and silently discard 27.00% of the log.

### Overview Table

| ID | Name | `type` | Class | Lines | Share | Responsibility |
|----|------|--------|:-----:|------:|------:|----------------|
| [001](001_assistant.md) | Assistant | `assistant` | A | 2,314,741 | 45.84% | Model turn — the API response message plus provenance and error accounting |
| [002](002_user.md) | User | `user` | A | 1,371,543 | 27.16% | User turn or tool result — two distinct things sharing one envelope |
| [003](003_attachment.md) | Attachment | `attachment` | A | 407,370 | 8.07% | Harness context injection — envelope for 23 distinct payload kinds |
| [004](004_last_prompt.md) | Last Prompt | `last-prompt` | B | 262,195 | 5.19% | Resume marker — the last prompt text and the leaf it attached to |
| [005](005_mode.md) | Mode | `mode` | B | 245,422 | 4.86% | Session mode transition |
| [006](006_ai_title.md) | AI Title | `ai-title` | B | 152,720 | 3.02% | Auto-generated conversation title |
| [007](007_permission_mode.md) | Permission Mode | `permission-mode` | B | 96,521 | 1.91% | Permission mode transition |
| [008](008_queue_operation.md) | Queue Operation | `queue-operation` | B | 76,222 | 1.51% | Command queued or dequeued while a turn was running |
| [009](009_system.md) | System | `system` | A | 45,201 | 0.895% | Lifecycle, telemetry, and error events — envelope for 10 distinct subtypes |
| [010](010_progress.md) | Progress | `progress` | A | 41,517 | 0.822% | Streaming progress for an in-flight tool or subagent — retired |
| [011](011_agent_name.md) | Agent Name | `agent-name` | B | 22,415 | 0.444% | Subagent display name |
| [012](012_file_history_snapshot.md) | File History Snapshot | `file-history-snapshot` | C | 8,016 | 0.159% | File state captured against a message, for checkpoint and undo |
| [013](013_custom_title.md) | Custom Title | `custom-title` | B | 4,276 | 0.085% | User-set conversation title |
| [014](014_pr_link.md) | PR Link | `pr-link` | B | 677 | 0.013% | Pull request associated with the session |
| [015](015_started.md) | Started | `started` | C | 329 | 0.0065% | Subagent invocation began |
| [016](016_result.md) | Result | `result` | C | 285 | 0.0056% | Subagent invocation completed |
| [017](017_summary.md) | Summary | `summary` | C | 178 | 0.0035% | Generated summary of a conversation thread |
| [018](018_fork_context_ref.md) | Fork Context Ref | `fork-context-ref` | C | 104 | 0.0021% | Back-reference from a forked session to its parent |
| [019](019_frame_link.md) | Frame Link | `frame-link` | B | 6 | 0.0001% | Association between a local path and a frame URL |

Instances are numbered by descending observed frequency. Counts sum to 5,049,738 — every parsed line in the store.

### Dispatch Levels

A consumer must dispatch up to three levels deep. This collection covers the first:

| Level | Discriminator | Distinct kinds | Collection |
|-------|---------------|---------------:|------------|
| Top-level envelope | `type` | 19 | **this collection** |
| Attachment payload | `attachment.type` when `type == "attachment"` | 23 | [`../attachment/`](../attachment/readme.md) |
| System event | `subtype` when `type == "system"` | 10 | [`../system_event/`](../system_event/readme.md) |

19 + 23 + 10 = **52 distinct event kinds** in total.

### Evidence Base

Every count, share, and presence rate in this collection derives from a full scan of the local session store:

| Property | Value |
|----------|-------|
| Session files scanned | 18,332 |
| Lines parsed | 5,049,738 |
| Unparseable lines | 37 (0.0007%) |
| Snapshot date | 2026-08-27 |
| Claude Code versions represented | 2.0.56 – 2.1.220 (20 distinct) |

Field types and presence rates come from a second, independent full pass over the same store. The store is live and append-only, so absolute counts drift upward between passes; ratios and the presence/absence contract do not.

**Store-range caveat**: the oldest data in this store is 2.0.56, so a kind observed across the full range has a `Since` floor of 2.0.56 — an artifact of the sample, not a claim about when the kind was introduced. Only a range starting or ending *strictly inside* 2.0.56 – 2.1.220 carries a real lifecycle signal.

### Type-Specific Requirements

All `envelope` doc instances must include:

1. **Title**: `# ENVELOPE: {Concept Name}` — using `ENVELOPE` as the type prefix
2. **Scope** (H3): 4 required bullets — Purpose, Responsibility, In Scope, Out of Scope
3. **Schema** (H3): discriminator line, field table with type and presence rate, and a real captured JSON example
4. **Notes** (H3): parsing considerations, presence anomalies, and known exceptions
5. **Since** (H3): observed version range with the store-range caveat applied
6. **Cross-References** (H3): flat table with `Type | File | Responsibility` columns

### Parsing Considerations

Dispatch on `type` **before** assuming any field is present:

- **Do not assume `uuid`**: 14 of 19 kinds never carry one — 869,366 lines (17.22%) lack it.
- **Do not assume `sessionId`**: 5 kinds never carry one and are unattributable to a session from the line alone.
- **Do not assume `version`**: only Class A kinds carry it, so 14 kinds cannot be attributed to a release.
- **Dispatch three levels deep**: `type` alone under-resolves `attachment` and `system`.
- **Unknown kinds**: retain or skip explicitly, never error — the taxonomy grows between releases, and `attachment` itself first appears in 2.1.197.

### Cross-Collection Dependencies

**This entity depends on**:
- [`../storage/001_projects_directory.md`](../storage/001_projects_directory.md) — file location and naming for session JSONL files
- [`../jsonl/`](../jsonl/readme.md) — conversation-entry field detail for the `user`/`assistant` kinds this collection only classifies

**This entity consumed by**:
- [`../../../../module/claude_storage_core/src/entry.rs`](../../../../module/claude_storage_core/src/entry.rs) — parser that currently accepts `user`/`assistant` only
- [`../../../../module/claude_storage/docs/invariant/003_entry_type_format.md`](../../../../module/claude_storage/docs/invariant/003_entry_type_format.md) — skip-handling contract; see [`../envelope_class/readme.md`](../envelope_class/readme.md) for a correction this taxonomy supplies

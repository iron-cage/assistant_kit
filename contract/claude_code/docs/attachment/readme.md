# Attachment Doc Entity

### Scope

- **Purpose**: Enumerate every `attachment.type` payload kind — the second dispatch level of the session log, and the channel through which Claude Code records what it injected into each turn's context.
- **Responsibility**: Master file for the `attachment` collection — one instance per payload kind, with its fields, presence rates, and contribution to context reconstruction.
- **In Scope**: All 23 observed `attachment.type` values; per-kind payload field tables with types and presence rates; observed frequency.
- **Out of Scope**: The `attachment` envelope itself (→ [`../envelope/003_attachment.md`](../envelope/003_attachment.md)); `system.subtype` values (→ [`../system_event/`](../system_event/readme.md)); the Class A field contract (→ [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md)).

**Discriminator**: `attachment.type`, on lines where the top-level `type` is `"attachment"`.

### Overview Table

| ID | Name | `attachment.type` | Lines | Share | Responsibility |
|----|------|-------------------|------:|------:|----------------|
| [001](001_total_tokens_reminder.md) | Total Tokens Reminder | `total_tokens_reminder` | 118,623 | 29.12% | Remaining context budget, as literal reminder text |
| [002](002_task_reminder.md) | Task Reminder | `task_reminder` | 70,130 | 17.22% | Current task-list state injected into the turn |
| [003](003_compact_file_reference.md) | Compact File Reference | `compact_file_reference` | 39,596 | 9.72% | A file cited across a compaction boundary |
| [004](004_deferred_tools_delta.md) | Deferred Tools Delta | `deferred_tools_delta` | 38,133 | 9.36% | Changes to the deferred-tool roster |
| [005](005_file.md) | File | `file` | 32,801 | 8.05% | Full file content injected into context |
| [006](006_skill_listing.md) | Skill Listing | `skill_listing` | 24,098 | 5.92% | Available-skill catalog |
| [007](007_agent_listing_delta.md) | Agent Listing Delta | `agent_listing_delta` | 17,559 | 4.31% | Changes to the available agent-type roster |
| [008](008_invoked_skills.md) | Invoked Skills | `invoked_skills` | 15,719 | 3.86% | Skills invoked this turn, with full bodies embedded |
| [009](009_ultrathink_effort.md) | Ultrathink Effort | `ultrathink_effort` | 14,322 | 3.52% | Marker that elevated reasoning effort was engaged |
| [010](010_queued_command.md) | Queued Command | `queued_command` | 10,141 | 2.49% | A command queued during a running turn |
| [011](011_command_permissions.md) | Command Permissions | `command_permissions` | 8,539 | 2.10% | Tool-permission grant scoped to a command |
| [012](012_mcp_instructions_delta.md) | MCP Instructions Delta | `mcp_instructions_delta` | 4,840 | 1.19% | Changes to MCP server instruction text |
| [013](013_date_change.md) | Date Change | `date_change` | 4,500 | 1.10% | Session crossed a calendar day boundary |
| [014](014_task_status.md) | Task Status | `task_status` | 3,613 | 0.887% | Background task lifecycle transition |
| [015](015_read_truncation_notice.md) | Read Truncation Notice | `read_truncation_notice` | 1,678 | 0.412% | A tool result was truncated before injection |
| [016](016_edited_text_file.md) | Edited Text File | `edited_text_file` | 1,591 | 0.391% | Post-edit file snippet |
| [017](017_plan_file_reference.md) | Plan File Reference | `plan_file_reference` | 1,352 | 0.332% | Plan file content injected into context |
| [018](018_plan_mode.md) | Plan Mode | `plan_mode` | 53 | 0.013% | Plan-mode reminder |
| [019](019_nested_memory.md) | Nested Memory | `nested_memory` | 31 | 0.0076% | A nested CLAUDE.md discovered and loaded |
| [020](020_plan_mode_exit.md) | Plan Mode Exit | `plan_mode_exit` | 28 | 0.0069% | Plan mode exited |
| [021](021_plan_mode_reentry.md) | Plan Mode Reentry | `plan_mode_reentry` | 15 | 0.0037% | Plan mode re-entered |
| [022](022_hook_additional_context.md) | Hook Additional Context | `hook_additional_context` | 7 | 0.0017% | Context contributed by a user hook |
| [023](023_context_tip.md) | Context Tip | `context_tip` | 1 | 0.0002% | One-off contextual tip |

Instances are numbered by descending observed frequency. Counts sum to exactly 407,370, matching the `attachment` envelope total in [`../envelope/003_attachment.md`](../envelope/003_attachment.md).

### The Context-Reconstruction Channel

Together these payload kinds record what a session's context window actually contained at any point, recoverable from the log alone with no API call or subprocess:

| Question | Kind |
|----------|------|
| How much budget is left? | [`total_tokens_reminder`](001_total_tokens_reminder.md) |
| Which tools are loaded vs. deferred? | [`deferred_tools_delta`](004_deferred_tools_delta.md) |
| Which agent types are available? | [`agent_listing_delta`](007_agent_listing_delta.md) |
| Which MCP servers contributed instructions? | [`mcp_instructions_delta`](012_mcp_instructions_delta.md) |
| Which skills exist, and which ran? | [`skill_listing`](006_skill_listing.md), [`invoked_skills`](008_invoked_skills.md) |
| What files were injected? | [`file`](005_file.md), [`nested_memory`](019_nested_memory.md), [`compact_file_reference`](003_compact_file_reference.md) |
| What is the task list? | [`task_reminder`](002_task_reminder.md), [`task_status`](014_task_status.md) |

**Delta kinds require a fold, not a lookup.** Three kinds carry *changes* rather than state. Only two of them mark a full roster:

| Kind | Has `isInitial` | Reconstruction |
|------|:---------------:|----------------|
| [`skill_listing`](006_skill_listing.md) | yes | fold forward from the most recent `isInitial: true` |
| [`agent_listing_delta`](007_agent_listing_delta.md) | yes | fold forward from the most recent `isInitial: true` |
| [`deferred_tools_delta`](004_deferred_tools_delta.md) | no | fold from the start of the session |
| [`mcp_instructions_delta`](012_mcp_instructions_delta.md) | no | fold from the start of the session |

Reading the newest line of any of these four yields a delta, not a roster.

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

All `attachment` doc instances must include:

1. **Title**: `# ATTACHMENT: {Concept Name}` — using `ATTACHMENT` as the type prefix
2. **Scope** (H3): 4 required bullets — Purpose, Responsibility, In Scope, Out of Scope
3. **Schema** (H3): discriminator line, field table with type and presence rate, and a real captured JSON example
4. **Notes** (H3): parsing considerations, presence anomalies, and known exceptions
5. **Since** (H3): observed version range with the store-range caveat applied
6. **Cross-References** (H3): flat table with `Type | File | Responsibility` columns

### Parsing Considerations

- **An empty payload is valid.** [`ultrathink_effort`](009_ultrathink_effort.md) carries only its `type` discriminator on all 14,322 occurrences. A parser requiring at least one field beyond the discriminator will reject it.
- **Rare kinds are the ones that break parsers.** [`context_tip`](023_context_tip.md) occurs once in 5,049,738 lines and [`hook_additional_context`](022_hook_additional_context.md) seven times. Their field sets are provisional.
- **Bodies are embedded, not referenced.** [`invoked_skills`](008_invoked_skills.md) and [`file`](005_file.md) carry full content, which dominates the cost of reading skill- and file-heavy sessions.
- **Truncation is signalled, not silent** — but only barely. [`read_truncation_notice`](015_read_truncation_notice.md) marks a truncated tool result, and `file.truncated` appears on 2 lines out of 32,801.

### Cross-Collection Dependencies

**This entity depends on**:
- [`../envelope/003_attachment.md`](../envelope/003_attachment.md) — the envelope carrying every payload in this collection
- [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) — Class A common-field contract the enclosing line satisfies

**This entity consumed by**:
- [`../tool/readme.md`](../tool/readme.md) — tool catalog referenced by `deferred_tools_delta` and `command_permissions`
- [`../behavior/036_b36_background_task_lifecycle.md`](../behavior/036_b36_background_task_lifecycle.md) — background-task lifecycle underlying `task_status`
- [`../behavior/033_b33_claudemd_loading_limits.md`](../behavior/033_b33_claudemd_loading_limits.md) — `CLAUDE.md` loading rules underlying `nested_memory`

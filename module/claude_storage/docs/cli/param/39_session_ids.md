# Parameter :: 39. `session_ids::`

### Scope

- **Purpose**: Specify the `session_ids::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `session_ids::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Comma-separated conversation selector for [`.cost`](../command/15_cost.md): full session IDs or unique ID prefixes, each resolved against every non-agent session across ALL projects.

**Type:** String (comma list)

**Fundamental Type:** String

**Constraints:**
- Split on `,`, each element trimmed; empty elements dropped. Splitting to zero non-empty IDs is an argument error (`session_ids must contain at least one session ID`), raised before any storage access
- Each element must be a full session ID (exact match wins) or a prefix matching exactly one distinct session ID — a prefix matching several is an error listing every match (sorted); an element matching nothing is an error naming it (`Session not found: <request>`)
- Resolution searches ALL projects, unlike [`session_id::`](14_session_id.md)'s single-command exact lookup and [`session::`](13_session.md)'s per-scope filtering — a conversation is addressable from anywhere, no `path::` needed
- An ID duplicated across project directories resolves to the copy with the greatest entry count (`Fix(BUG-528)` tie-break)
- Duplicate requests for one conversation collapse to the first occurrence; row order follows request order
- Agent (`agent-*`) sessions are never directly addressable — they fold into their root's row per [`agents::`](40_agents.md)

**Default:** none — when omitted, [`.cost`](../command/15_cost.md) reports the most recent non-agent session of the project owning [`path::`](09_path.md) (or the current directory), exiting `2` when no project or session exists

**Commands:** [`.cost`](../command/15_cost.md) — the only command registering this parameter.

**Purpose:** Selects which conversations the table reports on, one row per resolved conversation, in request order. Plural and cross-project by design — the single-selector precedents ([`session_id::`](14_session_id.md), [`session::`](13_session.md)) are scoped to one project or one exact ID and would not compose into a multi-row comparison table; a new parameter avoids overloading either with a third semantic.

**Examples:**
```bash
# One conversation by unique 8-char prefix
.cost session_ids::feed0011

# Compare two conversations — adds a TOTAL row
.cost session_ids::aaaa1111,bbbb2222

# Full IDs work anywhere prefixes do
.cost session_ids::feed0011-0000-4000-8000-000000000011
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| String | Base type | String | Comma list; each element an exact session ID or unique prefix |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 15 | [`.cost`](../command/15_cost.md) | most recent session of cwd's project | One row per resolved conversation, request order |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |

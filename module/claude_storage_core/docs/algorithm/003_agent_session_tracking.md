# Algorithm: Agent Session Tracking

### Scope

- **Purpose**: Classify whether a session file represents an agent/sub-conversation rather than a main user conversation.
- **Responsibility**: Documents the filename-based classification algorithm (`is_agent_session`) and why entry-level sidechain tagging is a deliberately separate, unimplemented concept (`is_agent_entry`).
- **In Scope**: Filename-based agent detection, its rationale, and the boundary with entry-level `isSidechain` data.
- **Out of Scope**: Associating agent session files with a root session (→ `claude_storage/docs/invariant/002_session_family.md`), continuation detection's own agent-filename skip rule (→ `../feature/004_continuation_detection.md`).

### Abstract

A session file is classified as an "agent session" (a sub-conversation spawned by a tool call) purely from its filename: files whose UUID stem starts with `agent-` are agent sessions, everything else is a main user session. This is the sole detection signal `Session::is_agent_session()` implements. Entry-level `isSidechain` tagging (present per-entry in the JSONL format) is a distinct concept that this library deliberately does not fold into session-level classification.

### Algorithm

`is_agent_session()` (`src/session.rs`):
1. Take the session's `id` (`SessionId` — the filename stem, without `.jsonl`).
2. Return `true` if the stem starts with the literal prefix `agent-`; `false` otherwise.

No entry content is loaded or inspected — the check is a synchronous, allocation-free string-prefix test on data the `Session` struct already holds, requiring no I/O.

**Why entry content is not consulted.** Each JSONL entry independently carries an `isSidechain: bool` field (`Entry::is_sidechain`), so the format could in principle mark sidechain conversations without an `agent-`-prefixed filename. A hypothetical entry-level check (`is_agent_entry`) was never implemented, for two reasons:
- It would require loading and parsing entries (I/O) into what is otherwise a cheap, synchronous, filename-only check.
- It depends on an `Entry::agent_id` field that was deliberately never added to the `Entry` struct.

In practice this loses no real detections: Claude Code always writes sidechain entries inside a file that also matches the `agent-*.jsonl` naming convention, so the filename check alone is sufficient for every session Claude Code itself produces.

**Consumers.** The boolean is propagated, never recomputed, by every caller that needs it:
- `SessionStats.is_agent_session` (`src/stats.rs`) — copied once via `session.stats()`.
- `SessionFilter.agent_only` (`src/filter.rs`) — `Session::matches_filter()` compares `is_agent_session()` against the requested `Option<bool>`.
- `cost::CostReport.is_agent_session` (`src/cost.rs`) — copied per report; `cost::agent_count` aggregates the count of `true` reports.
- `Project::stats()` (`src/project.rs`) — splits `agent_session_count` vs `main_session_count` across all sessions in a project.

### Edge Cases

| Scenario | Behavior |
|----------|----------|
| Filename stem starts with `agent-` (e.g. `agent-abc123.jsonl`) | `is_agent_session()` returns `true`, regardless of entry content |
| Filename stem does not start with `agent-`, entries carry `isSidechain: true` | `is_agent_session()` still returns `false` — entry content never widens the filename check (BUG-491 regression guard) |
| Filename stem does not start with `agent-`, no `isSidechain` entries | `is_agent_session()` returns `false` |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../src/session.rs` | `is_agent_session()` implementation |
| source | `../../src/entry.rs` | `Entry::is_sidechain` — the separate, entry-level signal not consulted here |
| source | `../../src/stats.rs` | `SessionStats.is_agent_session` — propagated, not recomputed |
| source | `../../src/cost.rs` | `CostReport.is_agent_session` and `agent_count` aggregation |
| source | `../../src/filter.rs` | `SessionFilter.agent_only` — filter surface for this classification |
| test | `../../tests/is_agent_session_doc_mismatch_bug.rs` | BUG-491 regression guard — locks in the filename-only contract |
| doc | `../feature/004_continuation_detection.md` | Continuation detection's own `agent-` filename skip rule (a separate consumer of the same naming convention) |

### Sources

| File | Notes |
|------|-------|
| `../../tests/is_agent_session_doc_mismatch_bug.rs` (BUG-491) | Bug reproducer that surfaced this missing doc — `is_agent_session()`'s own doc comment falsely claimed an unimplemented `isSidechain` OR-branch; this doc records the actual, corrected algorithm |

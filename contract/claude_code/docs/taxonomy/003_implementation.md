# Taxonomy: Implementation

### Scope

- **Purpose**: Document the current (1:1) and future (1:N) implementation mapping between Conversation and Session, and the `.projects` display implications.
- **Responsibility**: Authoritative instance for implementation differences — current Session Family definition, future Conversation Chain algorithm, `.projects` row semantics per era.
- **In Scope**: Current 1:1 Session Family mapping; future 1:N Conversation Chain algorithm (task 021); `.projects` row definition; Session UUID visibility rules.
- **Out of Scope**: Concept definitions (→ [001_concepts.md](001_concepts.md)); pairwise cardinalities (→ [002_relationships.md](002_relationships.md)).

### Current vs Future

| Aspect | Current (tasks 001–019) | Future (task 021+) |
|--------|------------------------|---------------------|
| Conversation = | Session Family (1:1 with root session file) | Conversation Chain (1:N with linked sessions) |
| Detection algorithm | N/A — each session is its own conversation | Chain detection: temporal proximity + content heuristics |
| `.projects` rows | One row per root session file | One row per detected conversation |
| Session visibility | Sessions shown as conversation rows | Sessions hidden; accessible at verbosity::2+ |

### Current Implementation: Session Family

In the current implementation, a **Conversation** corresponds to one **Session Family**: the root `.jsonl` file plus all agent sessions spawned from it (hierarchically or in flat layout).

**Session Family members:**
- Root session: `{uuid}.jsonl`
- Flat agent sessions: `agent-{id}.jsonl` siblings
- Hierarchical agent sessions: `{uuid}/subagents/agent-{id}.jsonl` children

All member sessions of a Session Family have the same parent context and represent a single user-visible Conversation.

The `.projects` command shows one row per Session Family (one row per root session file in the current implementation).

### Future Implementation: Conversation Chain

Task 021 introduces the **Conversation Chain** detection algorithm. Two or more consecutive sessions may represent the same logical conversation if:
- Their timestamps are in close temporal proximity
- Their content is topically related (same git branch, same project context)
- No explicit `--new-session` break between them

When chains are detected:
- Multiple Session Families are grouped under one Conversation
- `.projects` shows one row per detected Conversation (not per Session Family)
- Session UUIDs are accessible at `verbosity::2+` for debugging

### Session Visibility Rule

Session UUIDs appear only at higher verbosity levels (`verbosity::2+`) for debugging or direct access purposes. At default verbosity, `.projects` shows Conversations. This is intentional — UUIDs are not human-meaningful (`8d795a1c-c81d-4010-8d29-b4e678272419`).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Taxonomy master index: architecture diagram |
| taxonomy | [001_concepts.md](001_concepts.md) | Concept definitions: Project, Conversation, Session, Entry |
| taxonomy | [002_relationships.md](002_relationships.md) | Pairwise cardinalities |
| behavior | [`../behavior/018_b18_no_cross_session_links.md`](../behavior/018_b18_no_cross_session_links.md) | B18: no cross-session metadata — reason chain inference is needed |
| source | `../../../../module/claude_storage_core/docs/data_structure/001_storage_hierarchy.md` | Session Family data structure |

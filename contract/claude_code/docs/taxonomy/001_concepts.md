# Taxonomy: Concepts

### Scope

- **Purpose**: Define the four levels of the concept hierarchy used throughout `claude_storage` — Project, Conversation, Session, and Entry.
- **Responsibility**: Authoritative instance for concept definitions — what each level is, which layer it belongs to, and its storage correspondence.
- **In Scope**: Definition of all four levels; user-facing vs storage-layer classification; storage correspondence (directory, file, line).
- **Out of Scope**: Pairwise cardinalities (→ [002_relationships.md](002_relationships.md)); current vs future implementation mapping (→ [003_implementation.md](003_implementation.md)).

### Four Levels

| Level | Term | Layer | Description |
|-------|------|-------|-------------|
| 1 | **Project** | User-facing | A filesystem directory opened in Claude Code; groups all conversations for one workspace location |
| 2 | **Conversation** | User-facing | A logical interaction unit within a project; what the user experiences as "one chat" |
| 3 | **Session** | Storage | One `.jsonl` file on disk; the physical container written by Claude Code |
| 4 | **Entry** | Storage | One line in a `.jsonl` file; one turn in the conversation thread |

### User-Facing vs Storage

**User-facing layer**: Project and Conversation are the concepts users reason about. A user opens Claude Code in a project directory, starts a conversation, and sends messages. These are the visible units.

**Storage layer**: Session and Entry are storage implementation details. They exist on disk and are referenced internally, but users see conversations, not JSONL files. Surfacing session IDs in primary output adds noise without value:
- Session IDs are UUIDs: `8d795a1c-c81d-4010-8d29-b4e678272419` — not human-meaningful
- Multiple sessions may represent the same logical conversation (chain)
- Agent session IDs are even more opaque: `agent-a6061d6e2a0c37a78`

### Storage Correspondence

| Term | Storage Object | Location |
|------|---------------|----------|
| Project | Directory | `~/.claude/projects/{encoded-path}/` or `~/.claude/projects/{uuid}/` |
| Conversation | Logical grouping | No physical artifact — corresponds to one or more Sessions |
| Session | `.jsonl` file | `~/.claude/projects/{project-id}/{session-id}.jsonl` |
| Entry | One JSON line | A single line within the `.jsonl` file |

### Zero-Entry Sessions

Sessions with zero entries can exist as startup placeholders (B8 — Claude Code creates an empty file before writing the first entry; the file remains at 0 bytes if the process exits without writing).

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Taxonomy master index: architecture diagram |
| taxonomy | [002_relationships.md](002_relationships.md) | All six pairwise cardinalities |
| taxonomy | [003_implementation.md](003_implementation.md) | Current 1:1 vs future 1:N Conversation/Session mapping |
| storage | [`../storage/readme.md`](../storage/readme.md) | Physical storage layout and containment model |
| behavior | [`../behavior/008_b8_zero_byte_placeholder.md`](../behavior/008_b8_zero_byte_placeholder.md) | Zero-byte session placeholder behavior |

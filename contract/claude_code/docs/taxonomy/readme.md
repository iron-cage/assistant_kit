# Taxonomy

### Scope

- **Purpose**: Define the four-level concept hierarchy (Project / Conversation / Session / Entry) used throughout `claude_storage` and its CLI.
- **Responsibility**: Master file for the `taxonomy` doc entity — lists all 3 taxonomy instances covering the concept definitions, pairwise relationships, and current vs future implementation mapping.
- **In Scope**: Concept definitions, cardinality rules, pairwise relationship descriptions, containment diagram, user-facing vs storage-layer distinction, current and future implementation notes.
- **Out of Scope**: Concrete field schemas (→ [`../jsonl/`](../jsonl/readme.md)); filesystem paths and directory layout (→ [`../storage/`](../storage/readme.md)); CLI command reference (→ `../../../../module/claude_storage/docs/cli/`).

### Overview Table

| ID | Name | Responsibility |
|----|------|----------------|
| [001](001_concepts.md) | Concepts | The four levels — Project, Conversation, Session, Entry — with definitions, layer assignment, and storage correspondence |
| [002](002_relationships.md) | Relationships | All six pairwise cardinalities (Project↔Conversation, Conversation↔Session, Session↔Entry, transitive pairs) with containment diagram |
| [003](003_implementation.md) | Implementation | Current 1:1 Conversation/Session mapping vs future N:1 Conversation Chain algorithm (task 021); `.projects` row definition per era |

### Four Levels at a Glance

| Level | Term | Layer | Description |
|-------|------|-------|-------------|
| 1 | **Project** | User-facing | A filesystem directory opened in Claude Code; groups all conversations for one workspace location |
| 2 | **Conversation** | User-facing | A logical interaction unit within a project; what the user experiences as "one chat" |
| 3 | **Session** | Storage | One `.jsonl` file on disk; the physical container written by Claude Code |
| 4 | **Entry** | Storage | One line in a `.jsonl` file; one turn in the conversation thread |

**User-facing vs storage**: Project and Conversation are the concepts users reason about. Session and Entry are storage implementation details — they exist on disk but users see conversations, not JSONL files.

### Architecture Diagram

```
╔═══════════════════════════════════════════════════════════════════════╗
║                      USER-FACING LAYER                                ║
║   ┌─────────────────────────────────────────────────────────────┐    ║
║   │                        PROJECT                              │    ║
║   │   filesystem directory opened in Claude Code                │    ║
║   │   ~/.claude/projects/{path-encoded}/                        │    ║
║   └───────────────────────────┬─────────────────────────────────┘    ║
║                               │ 1:N contains                          ║
║   ┌───────────────────────────▼─────────────────────────────────┐    ║
║   │                      CONVERSATION                           │    ║
║   │   logical interaction unit — "one chat" from user's view    │    ║
║   └─────────────────────────────────────────────────────────────┘    ║
╠═══════════════════════════════════════════════════════════════════════╣
║   boundary: users see conversations — sessions are hidden below       ║
╠═══════════════════════════════════════════════════════════════════════╣
║                       STORAGE LAYER                                   ║
║   ┌─────────────────────────────────────────────────────────────┐    ║
║   │                        SESSION                              │    ║
║   │   one .jsonl file on disk — physical container              │    ║
║   └───────────────────────────┬─────────────────────────────────┘    ║
║                               │ 1:N contains                          ║
║   ┌───────────────────────────▼─────────────────────────────────┐    ║
║   │                         ENTRY                               │    ║
║   │   one line in .jsonl — one turn (user or assistant message) │    ║
║   └─────────────────────────────────────────────────────────────┘    ║
╚═══════════════════════════════════════════════════════════════════════╝
```

### Type-Specific Requirements

All `taxonomy` doc instances must include:

1. **Title**: `# Taxonomy: {Group Name}` — using `Taxonomy` as the type prefix
2. **Scope** (H3): 4 required bullets — Purpose, Responsibility, In Scope, Out of Scope
3. **Content** (H3): Definitions, cardinalities, diagrams, or implementation notes for this group
4. **Cross-References** (H3): Flat table with `Type | File | Responsibility` columns

### Cross-Doc Entity Dependencies

**This entity depends on**:
- `../storage/` — Session and Entry storage correspondence
- `../jsonl/` — Entry-level JSONL schema details
- `../behavior/` — B17, B18 cross-session boundary behaviors that motivate the Conversation abstraction

**This entity consumed by**:
- `../../../../module/claude_storage/docs/cli/` — CLI dictionary and commands reference this taxonomy
- `../../../../module/claude_storage_core/docs/data_structure/001_storage_hierarchy.md` — Session Family data structure
- `../../../../module/claude_storage/docs/` — storage implementation docs use these concepts throughout

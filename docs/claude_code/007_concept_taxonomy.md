# Claude Code: Concept Taxonomy

### Scope

- **Purpose**: Define the four-level concept hierarchy (Project / Conversation / Session / Entry) used throughout `claude_storage` and its CLI, with pairwise relationship descriptions and containment diagram.
- **Responsibility**: Single source of truth for terminology boundaries — which concept belongs to the user-facing layer vs the storage-layer, and how each pair relates.
- **In Scope**: Concept definitions, cardinality rules, pairwise relationship descriptions, containment diagram, storage correspondence for each level, user-facing vs storage-layer distinction.
- **Out of Scope**: Concrete field schemas (→ [004_jsonl_format.md](004_jsonl_format.md)); filesystem paths and directory layout (→ [002_storage_organization.md](002_storage_organization.md)); CLI command reference (→ `module/claude_storage/docs/cli/`).

---

### Four Levels

| Level | Term | Layer | Description |
|-------|------|-------|-------------|
| 1 | **Project** | User-facing | A filesystem directory opened in Claude Code; groups all conversations for one workspace location |
| 2 | **Conversation** | User-facing | A logical interaction unit within a project; what the user experiences as "one chat" |
| 3 | **Session** | Storage | One `.jsonl` file on disk; the physical container written by Claude Code |
| 4 | **Entry** | Storage | One line in a `.jsonl` file; one turn in the conversation thread |

**User-facing vs storage**: Project and Conversation are the concepts users reason about. Session and Entry are storage implementation details — they exist on disk and are referenced internally, but users see conversations, not JSONL files.

---

### Containment Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│  Storage Root  (~/.claude/projects/)                                │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  Project  (-home-user1-pro/)                                 │  │
│  │                                                              │  │
│  │  ┌──────────────────────────┐  ┌──────────────────────────┐ │  │
│  │  │  Conversation 1          │  │  Conversation 2          │ │  │
│  │  │  ┌──────────────────┐    │  │  ┌────────┐  ┌────────┐  │ │  │
│  │  │  │  Session A       │    │  │  │Session │  │Session │  │ │  │
│  │  │  │  (root .jsonl)   │    │  │  │   B    │  │   C    │  │ │  │
│  │  │  │  Entry 1         │    │  │  │        │  │        │  │ │  │
│  │  │  │  Entry 2  ...    │    │  │  │ (chain │  │ of B)  │  │ │  │
│  │  │  └──────────────────┘    │  │  └────────┘  └────────┘  │ │  │
│  │  │  ┌──────────────────┐    │  │                           │ │  │
│  │  │  │  Agent Session   │    │  └──────────────────────────┘ │  │
│  │  │  │  (agent-*.jsonl) │    │                               │  │
│  │  │  │  Entry 1 ...     │    │                               │  │
│  │  │  └──────────────────┘    │                               │  │
│  │  └──────────────────────────┘                               │  │
│  └──────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

**Note on Conversation 2 (Sessions B + C)**: A conversation spanning multiple session files requires the Conversation Chain detection algorithm (task 021). In the current implementation, each session = one conversation (Sessions B and C are two separate conversations until the algorithm is deployed).

---

### Pairwise Relationships

#### Project ↔ Conversation

| Aspect | Description |
|--------|-------------|
| Cardinality | One Project contains **1..N** Conversations; each Conversation belongs to **exactly one** Project |
| Direction | Project groups all conversations at the same filesystem path |
| Access | `.projects` lists conversations grouped by project |
| Isolation | Conversations from different projects never share sessions or entries |

A Project is the organizational unit; Conversation is the interaction unit. A project with many `--new-session` invocations accumulates many conversations.

#### Conversation ↔ Session

| Aspect | Description |
|--------|-------------|
| Cardinality | One Conversation contains **1..N** Sessions; each Session belongs to **exactly one** Conversation |
| Direction | Conversation is the logical grouping; Session is the physical file |
| Current implementation | 1:1 — one Session Family = one Conversation |
| Future implementation | 1:N — session chain detection groups multiple consecutive sessions into one Conversation |
| User visibility | Users see Conversations; Sessions are the storage detail surfaced only at higher verbosity levels |

The key insight: default Claude Code behavior (`claude` with no `--new-session`) appends to the **same** JSONL file — one session, one growing conversation. When `--new-session` is used or a new run creates a separate file, two sessions may represent the same logical conversation continued across runs. The Conversation Chain algorithm (task 021) detects and groups these.

#### Session ↔ Entry

| Aspect | Description |
|--------|-------------|
| Cardinality | One Session contains **0..N** Entries; each Entry belongs to **exactly one** Session |
| Direction | Session is the container; Entry is the atomic unit |
| Threading | Entries within a session are linked by `parentUuid` into a single singly-linked chain |
| Cross-session links | **None** — `parentUuid` never references a UUID from a different session file (B17) |
| Append-only | Entries are never modified after being written |

Zero-entry sessions can exist as startup placeholders (B8 — Claude Code creates an empty file before writing the first entry; the file remains at 0 bytes if the process exits without writing).

#### Project ↔ Session

| Aspect | Description |
|--------|-------------|
| Cardinality | One Project contains **1..N** Sessions (across all its conversations); each Session belongs to one Project (via its Conversation) |
| Storage path | Sessions are files in `~/.claude/projects/{path-encoded}/` |
| Session types | Main sessions (root JSONL) and agent sessions (agent-*.jsonl or subagents/agent-*.jsonl) |
| No cross-project sessions | A session file always lives inside exactly one project directory |

#### Project ↔ Entry

| Aspect | Description |
|--------|-------------|
| Cardinality | One Project contains **0..M** Entries (across all sessions and conversations) |
| Access | Entries are not directly addressable by project — access goes Project → Session → Entry |
| Counting | `.count target::entries project::P` gives the total entry count for a project |

#### Conversation ↔ Entry

| Aspect | Description |
|--------|-------------|
| Cardinality | One Conversation contains **0..M** Entries (across its Sessions) |
| Threading | Within a session, entries are linked by `parentUuid`; across sessions in a chain, no explicit thread link exists |
| Agent entries | Entries in agent sessions belong to the same Conversation as their root session, as part of the Session Family |

---

### Why Sessions Are a Hidden Detail

Session UUIDs and file names are storage artifacts, not user identifiers. The user interacts with Claude Code and gets a conversation — they don't think in terms of which `.jsonl` file was created. Surfacing session IDs in primary output adds noise without adding value:

- Session IDs are UUIDs: `8d795a1c-c81d-4010-8d29-b4e678272419` — not human-meaningful
- Multiple sessions may represent the same logical conversation (chain)
- Agent session IDs are even more opaque: `agent-a6061d6e2a0c37a78`

The `.projects` command therefore uses Conversation as the display unit. Session UUIDs appear only at higher verbosity levels (`verbosity::2+`) for debugging or direct access purposes.

---

### Current vs Future Implementation

| Aspect | Current (tasks 001–019) | Future (task 021+) |
|--------|------------------------|---------------------|
| Conversation = | Session Family (1:1 with root session file) | Conversation Chain (1:N with linked sessions) |
| Detection algorithm | N/A — each session is its own conversation | Chain detection: temporal proximity + content heuristics |
| `.projects` rows | One row per root session file | One row per detected conversation |
| Session visibility | Sessions shown as conversation rows | Sessions hidden; accessible at verbosity::2+ |

---

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| Storage layout | [002_storage_organization.md](002_storage_organization.md) | Filesystem directory structure and containment diagram |
| Entry schema | [004_jsonl_format.md](004_jsonl_format.md) | JSONL field definitions and threading model |
| Session behaviors | [001_session_behaviors.md](001_session_behaviors.md) | Observed behaviors including B17 (no cross-session parentUuid) and B18 (no continuation metadata) |
| CLI dictionary | `module/claude_storage/docs/cli/dictionary.md` | Canonical term definitions for CLI documentation |
| CLI commands | `module/claude_storage/docs/cli/commands.md` | `.projects`, `.list`, `.count`, `.show` command reference |

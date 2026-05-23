# Taxonomy: Relationships

### Scope

- **Purpose**: Document all six pairwise cardinalities between the four concept levels.
- **Responsibility**: Authoritative instance for pairwise relationship descriptions — cardinality, direction, isolation, and access patterns for each pair.
- **In Scope**: All six pairs (Project↔Conversation, Conversation↔Session, Session↔Entry, and three transitive pairs); containment diagram with agent sessions and chains.
- **Out of Scope**: Concept definitions (→ [001_concepts.md](001_concepts.md)); current vs future implementation (→ [003_implementation.md](003_implementation.md)).

### All Six Pairwise Cardinalities

```
  Project      ──1:N──▶  Conversation   one workspace, many chats
  Project      ──1:N──▶  Session        transitive via Conversation
  Project      ──1:M──▶  Entry          transitive via Session

  Conversation ──1:1──▶  Session        now   — one .jsonl = one chat
  Conversation ──1:N──▶  Session        future — chain of .jsonl files (task 021)
  Conversation ──1:M──▶  Entry          transitive via Session

  Session      ──1:N──▶  Entry          one file, many turns
```

### Project ↔ Conversation

| Aspect | Description |
|--------|-------------|
| Cardinality | One Project contains **1..N** Conversations; each Conversation belongs to **exactly one** Project |
| Direction | Project groups all conversations at the same filesystem path |
| Access | `.projects` lists conversations grouped by project |
| Isolation | Conversations from different projects never share sessions or entries |

### Conversation ↔ Session

| Aspect | Description |
|--------|-------------|
| Cardinality | One Conversation contains **1..N** Sessions; each Session belongs to **exactly one** Conversation |
| Current implementation | 1:1 — one Session Family = one Conversation |
| Future implementation | 1:N — Conversation Chain groups multiple consecutive sessions |
| User visibility | Users see Conversations; Sessions are the storage detail surfaced only at higher verbosity |

When `--continue` / `-c` is passed, `claude` appends to the **same** JSONL file — one session, one growing conversation. When a new session starts (invocation without `--continue`), a new JSONL file is created. Two consecutive sessions may represent the same logical conversation continued across runs — the Conversation Chain algorithm (task 021) detects this.

### Session ↔ Entry

| Aspect | Description |
|--------|-------------|
| Cardinality | One Session contains **0..N** Entries; each Entry belongs to **exactly one** Session |
| Threading | Entries within a session are linked by `parentUuid` into a singly-linked chain |
| Cross-session links | **None** — `parentUuid` never references a UUID from a different session file (B17) |
| Append-only | Entries are never modified after being written |

### Containment Diagram (with Agent Sessions and Chains)

```
~/.claude/projects/
└── -home-alice-projects/                    ← Project
    │
    ├── [Conversation 1]  ─────────────── user-facing grouping
    │   ├── a1b2c3d4.jsonl              ← Session A (root)
    │   │   ├── Entry (user)
    │   │   ├── Entry (assistant)
    │   │   └── ...
    │   └── agent-e5f6.jsonl            ← Agent Session (part of Conv 1)
    │       └── ...
    │
    └── [Conversation 2]  ─────────────── future: chain of 2 sessions
        ├── 7e8f9a0b.jsonl              ← Session B  ─┐ chained by
        │   └── ...                                    │ temporal proximity
        └── c1d2e3f4.jsonl              ← Session C  ─┘ (task 021)
            └── ...
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Taxonomy master index: architecture diagram |
| taxonomy | [001_concepts.md](001_concepts.md) | Concept definitions for all four levels |
| taxonomy | [003_implementation.md](003_implementation.md) | Current 1:1 vs future 1:N Conversation/Session mapping |
| behavior | [`../behavior/018_b18_no_cross_session_links.md`](../behavior/018_b18_no_cross_session_links.md) | No cross-session continuation metadata (basis for chain inference) |

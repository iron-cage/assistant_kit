# JSONL: Threading Model

### Scope

- **Purpose**: Specify the conversation threading model — how JSONL entries are linked via `parentUuid` into a singly-linked chain.
- **Responsibility**: Authoritative instance for the threading model — chain structure, null root, self-containment guarantee, and compaction exception.
- **In Scope**: `parentUuid` chain structure; null root; singly-linked forward traversal; self-containment per session; compaction boundary exception.
- **Out of Scope**: `parentUuid` field definition (→ [001_common_fields.md](001_common_fields.md)); cross-session link absence (→ [`../behavior/018_b18_no_cross_session_links.md`](../behavior/018_b18_no_cross_session_links.md)).

### Threading Structure

Entries are linked via `parentUuid` into a singly-linked chain rooted at `null`:

```
Entry 1 (User):      uuid=A, parentUuid=null
  ↓
Entry 2 (Assistant): uuid=B, parentUuid=A
  ↓
Entry 3 (User):      uuid=C, parentUuid=B
  ↓
Entry 4 (Assistant): uuid=D, parentUuid=C
```

**Properties:**
- **Root**: First entry in a session has `parentUuid: null`
- **Forward-linked**: Each entry points to its predecessor (not successor)
- **Singly-linked**: One parent per entry; no branching
- **Self-contained**: All referenced UUIDs exist as `uuid` fields within the same `.jsonl` file (with compaction exception)

### Self-Containment Guarantee

The `parentUuid` chain within one session file is closed for the vast majority of entries: no entry references a UUID that lives in a different file (behavior B17).

**Exception at context-compaction boundaries**: When Claude Code exhausts the context window and resumes, the continuation user message may have a `parentUuid` referencing a UUID from the pre-compaction context that was never written as a top-level entry. These orphaned references occur at < 0.2% of all non-null `parentUuid` entries.

### Cross-Session Boundary

New sessions start with `parentUuid: null` — no field references the prior session. Cross-session conversation chains must be inferred by heuristics (temporal proximity, content context). See behavior B18 and [`../taxonomy/003_implementation.md`](../taxonomy/003_implementation.md).

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | JSONL master index |
| jsonl | [001_common_fields.md](001_common_fields.md) | `parentUuid` field definition |
| behavior | [`../behavior/010_b10_entry_threading.md`](../behavior/010_b10_entry_threading.md) | B10: entry threading behavior |
| behavior | [`../behavior/017_b17_parentuuid_self_contained.md`](../behavior/017_b17_parentuuid_self_contained.md) | B17: self-containment rule and compaction exception |
| behavior | [`../behavior/018_b18_no_cross_session_links.md`](../behavior/018_b18_no_cross_session_links.md) | B18: no cross-session continuation metadata |
| taxonomy | [`../taxonomy/003_implementation.md`](../taxonomy/003_implementation.md) | Conversation Chain detection algorithm |

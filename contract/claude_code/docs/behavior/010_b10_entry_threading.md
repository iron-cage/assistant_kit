# Behavior B10: Entry Threading via parentUuid

### Scope

- **Purpose**: Document that JSONL entries are linked by `parentUuid` into a singly-linked chain with null root.
- **Responsibility**: Authoritative instance for behavior B10 — defines the threading model, certainty level, and supporting evidence.
- **In Scope**: `parentUuid` chain structure; null root entry; singly-linked forward chain.
- **Out of Scope**: Self-containment constraint within one session file (→ [B17](017_b17_parentuuid_self_contained.md)); cross-session link absence (→ [B18](018_b18_no_cross_session_links.md)); full entry schema (→ [`../jsonl/009_threading_model.md`](../jsonl/009_threading_model.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 95% | **Tier**: VALIDATED | **Evidence**: E9, E20

Each JSONL entry contains a `parentUuid` field that links it to the previous entry in the conversation. The first entry has `"parentUuid": null`. This forms a singly-linked chain that can be walked to reconstruct conversation order:

```
Entry 1 (User):      uuid=A, parentUuid=null
  ↓
Entry 2 (Assistant): uuid=B, parentUuid=A
  ↓
Entry 3 (User):      uuid=C, parentUuid=B
  ↓
Entry 4 (Assistant): uuid=D, parentUuid=C
```

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E9 | B10 | Doc | `../jsonl/009_threading_model.md` | Threading model | `parentUuid` links each entry to its parent; null on first entry of a thread |
| E20 | B10 | Test | `../../tests/behavior/b10_entry_threading.rs` | `b10_first_entry_has_null_parent_uuid`, `b10_subsequent_entries_have_non_null_parent_uuid` | First conversation entry has `parentUuid:null`; second has non-null `parentUuid` referencing first |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [017_b17_parentuuid_self_contained.md](017_b17_parentuuid_self_contained.md) | `parentUuid` chain stays within one session file |
| behavior | [018_b18_no_cross_session_links.md](018_b18_no_cross_session_links.md) | No cross-session continuation pointers |
| jsonl | [`../jsonl/009_threading_model.md`](../jsonl/009_threading_model.md) | Full threading model schema |
| test | `../../tests/behavior/b10_entry_threading.rs` | Invalidation test |

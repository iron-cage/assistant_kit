# Behavior B18: No Cross-Session Continuation Metadata

### Scope

- **Purpose**: Document that no cross-session continuation metadata is written — new sessions always start with `parentUuid: null`.
- **Responsibility**: Authoritative instance for behavior B18 — defines the storage boundary rule, implication for conversation chain inference, certainty level, and supporting evidence.
- **In Scope**: `parentUuid: null` as first entry in every new session; no field referencing prior session; conversation chain inference requirement.
- **Out of Scope**: Self-containment within a session (→ [B17](017_b17_parentuuid_self_contained.md)); Conversation Chain algorithm definition (→ [`../taxonomy/003_implementation.md`](../taxonomy/003_implementation.md)).

### Behavior

**Status**: 🎯 Observed | **Certainty**: 80% | **Tier**: VALIDATED | **Evidence**: E34

When Claude Code starts a new session in a project that already has sessions (whether via a fresh invocation without `--continue`, or via `--new-session` in the `clr` wrapper), the first entry of the new session has `parentUuid: null`. No field in the new session's entries references the prior session's UUID or last entry UUID.

This means:
- Two consecutive sessions in the same project directory look identical from a storage perspective whether they are logically connected or not
- Grouping sessions into Conversations (Conversation Chains) requires heuristic inference — temporal proximity, content context, or external markers
- Claude Code itself has no "resume from prior conversation" semantic in storage; it only "continue current session" (append to same file) or "start new" (create a new file)

See [`../taxonomy/003_implementation.md`](../taxonomy/003_implementation.md) for how Conversation Chains are defined relative to this storage reality.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E34 | B18 | Test | `../../tests/behavior/b18_no_cross_session_links.rs` | `it_first_entry_parentuuid_is_null` | First conversation entry (user or assistant type) in each session has `parentUuid: null` or absent — no cross-session continuation pointer written |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [017_b17_parentuuid_self_contained.md](017_b17_parentuuid_self_contained.md) | `parentUuid` self-containment within session file |
| taxonomy | [`../taxonomy/003_implementation.md`](../taxonomy/003_implementation.md) | Conversation Chain detection algorithm (task 021) |
| test | `../../tests/behavior/b18_no_cross_session_links.rs` | Invalidation test |

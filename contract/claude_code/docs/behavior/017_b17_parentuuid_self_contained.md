# Behavior B17: parentUuid Chain Is Self-Contained Per Session

### Scope

- **Purpose**: Document that `parentUuid` references within one session file are self-contained, with a known compaction boundary exception.
- **Responsibility**: Authoritative instance for behavior B17 — defines the self-containment rule, compaction exception, certainty level, and supporting evidence.
- **In Scope**: Self-containment of `parentUuid` within one `.jsonl` file; context-compaction boundary exception; < 0.2% violation rate.
- **Out of Scope**: Cross-session link absence (→ [B18](018_b18_no_cross_session_links.md)); entry threading model (→ [B10](010_b10_entry_threading.md)).

### Behavior

**Status**: 🎯 Observed | **Certainty**: 85% | **Tier**: VALIDATED | **Since**: pre-v1.0 | **Evidence**: E33

Within one `.jsonl` session file, the `parentUuid` threading is closed for the vast majority of entries — no entry references a UUID that lives in a different file.

**Known exception — context-compaction boundaries:** When Claude Code's context window is exhausted and the conversation is resumed, the continuation user message is appended to the existing `.jsonl` with a `parentUuid` that references the last UUID from the pre-compaction context. That UUID may have existed only in the previous context window and was never written into the file as a top-level `uuid` entry; the orphaned reference is expected and unavoidable. Empirically, these violations are rare (< 0.2% of entries with a non-null `parentUuid`).

This is why cross-session conversation chains must be inferred: for B17-conforming entries there is no pointer to jump to, and for the small number of compaction-boundary exceptions the pointer is dangling. The boundary between two sessions is a hard storage boundary.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E33 | B17 | Test | `../../tests/behavior/b17_parentuuid_self_contained.rs` | `it_parentuuid_never_crosses_session_boundary` | Rate-based check: orphaned `parentUuid` references stay below 1% across 10 projects × 5 sessions |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [010_b10_entry_threading.md](010_b10_entry_threading.md) | Entry threading model via `parentUuid` |
| behavior | [018_b18_no_cross_session_links.md](018_b18_no_cross_session_links.md) | No cross-session continuation metadata (new session starts with `parentUuid: null`) |
| jsonl | [`../jsonl/009_threading_model.md`](../jsonl/009_threading_model.md) | Threading model spec: compaction exception detail |
| test | `../../tests/behavior/b17_parentuuid_self_contained.rs` | Invalidation test |

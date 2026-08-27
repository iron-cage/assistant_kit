# ENVELOPE CLASS: Full Envelope

### Scope

- **Purpose**: Define Class A — the five top-level kinds carrying all nine common fields, and the only kinds attributable to a Claude Code release from the line alone.
- **Responsibility**: Authoritative instance for the Class A field contract, its membership, and the correlation capabilities that contract enables.
- **In Scope**: Membership, the guaranteed-present field set, threading and version attribution, and the parsing consequences of Class A's completeness.
- **Out of Scope**: Per-kind payload semantics (→ [`../envelope/`](../envelope/readme.md)); the internal shape of `user`/`assistant` content (→ [`../jsonl/`](../jsonl/readme.md)); the other two classes (→ [002](002_session_scoped.md), [003](003_detached.md)).

### Membership

Five kinds, 4,180,372 lines — 82.78% of the store:

| Kind | Envelope Instance | Lines | Share of store |
|------|-------------------|------:|---------------:|
| `assistant` | [001_assistant.md](../envelope/001_assistant.md) | 2,314,741 | 45.84% |
| `user` | [002_user.md](../envelope/002_user.md) | 1,371,543 | 27.16% |
| `attachment` | [003_attachment.md](../envelope/003_attachment.md) | 407,370 | 8.07% |
| `system` | [009_system.md](../envelope/009_system.md) | 45,201 | 0.90% |
| `progress` | [010_progress.md](../envelope/010_progress.md) | 41,517 | 0.82% |

### Field Contract

All nine common fields are present on 100% of Class A lines:

| Field | Type | Role |
|-------|------|------|
| `uuid` | string | The line's own identity; the join target for every correlation handle in [003](003_detached.md) |
| `parentUuid` | string \| null | Thread link to the preceding entry |
| `timestamp` | string | ISO-8601 event time |
| `sessionId` | string | Owning session |
| `cwd` | string | Working directory at the time of the entry |
| `version` | string | Claude Code release that wrote the line |
| `gitBranch` | string | Branch checked out at the time of the entry |
| `userType` | string | Actor classification |
| `isSidechain` | boolean | Whether the entry belongs to a sidechain rather than the main thread |

Full field-level detail for these nine is specified in [`../jsonl/001_common_fields.md`](../jsonl/001_common_fields.md); that document's contract applies to exactly this class and no other.

### Correlation

Class A is the only class that supports correlation without external context:

- **By identity** — `uuid` is the target every Class C handle resolves against ([003](003_detached.md)).
- **By thread** — `parentUuid` chains entries into a conversation. The chain breaks at a compaction boundary and is repaired by `logicalParentUuid` on [`compact_boundary`](../system_event/001_compact_boundary.md).
- **By session** — `sessionId` attributes a line without reference to the file it was found in.
- **By release** — `version` is the sole in-log attribution to a Claude Code release.
- **By time** — `timestamp` orders Class A lines against each other and against the three Class B kinds that carry one.

### Notes

**This is the only class carrying `version`**, which is what makes lifecycle analysis possible at all. The observed version range of a Class B or C kind cannot be read from its own lines — see [002](002_session_scoped.md) and [003](003_detached.md).

**Class A is where the taxonomy grows.** `attachment` entered the class in 2.1.197 and `progress` left it after 2.1.81. Both transitions occurred strictly inside the store's version span, so both are genuine lifecycle signals rather than sampling artifacts.

**Membership does not imply parseability.** [`claude_storage_core`](../../../../module/claude_storage_core/src/entry.rs) accepts only `user` and `assistant`; the other three Class A kinds satisfy the full common-field contract and are still rejected at the type check. Class A is a statement about fields, not about consumer support.

**Two levels of further dispatch live inside this class.** `attachment` resolves 23 ways and `system` 10 ways — see [`../attachment/`](../attachment/readme.md) and [`../system_event/`](../system_event/readme.md). A consumer that dispatches on `type` alone has resolved Class A only partially.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| envelope_class | [readme.md](readme.md) | Class master index, presence matrix, and evidence base |
| envelope_class | [002_session_scoped.md](002_session_scoped.md) | Class B — `sessionId` and payload only |
| envelope_class | [003_detached.md](003_detached.md) | Class C — no common fields, correlation by handle |
| envelope | [`../envelope/readme.md`](../envelope/readme.md) | All 19 top-level kinds this class partitions |
| jsonl | [`../jsonl/001_common_fields.md`](../jsonl/001_common_fields.md) | Field-level schema for the nine common fields |
| jsonl | [`../jsonl/009_threading_model.md`](../jsonl/009_threading_model.md) | `parentUuid` threading, available only on this class |
| behavior | [`../behavior/017_b17_parentuuid_self_contained.md`](../behavior/017_b17_parentuuid_self_contained.md) | Self-containment rule governing `parentUuid` |
| source | [`../../../../module/claude_storage_core/src/entry.rs`](../../../../module/claude_storage_core/src/entry.rs) | Parser accepting two of this class's five kinds |

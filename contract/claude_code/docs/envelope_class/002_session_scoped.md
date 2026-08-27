# ENVELOPE CLASS: Session-Scoped

### Scope

- **Purpose**: Define Class B — the nine top-level kinds carrying `sessionId` and their own payload, and nothing else.
- **Responsibility**: Authoritative instance for the Class B field contract, its membership, the `timestamp` split within it, and the correlation limits that follow.
- **In Scope**: Membership, the guaranteed-present and guaranteed-absent field sets, the three-member `timestamp` subset, and the parsing consequences of Class B's minimalism.
- **Out of Scope**: Per-kind payload semantics (→ [`../envelope/`](../envelope/readme.md)); the other two classes (→ [001](001_full_envelope.md), [003](003_detached.md)).

### Membership

Nine kinds, 860,454 lines — 17.04% of the store:

| Kind | Envelope Instance | Lines | `timestamp` |
|------|-------------------|------:|:-----------:|
| `last-prompt` | [004_last_prompt.md](../envelope/004_last_prompt.md) | 262,195 | ❌ |
| `mode` | [005_mode.md](../envelope/005_mode.md) | 245,422 | ❌ |
| `ai-title` | [006_ai_title.md](../envelope/006_ai_title.md) | 152,720 | ❌ |
| `permission-mode` | [007_permission_mode.md](../envelope/007_permission_mode.md) | 96,521 | ❌ |
| `queue-operation` | [008_queue_operation.md](../envelope/008_queue_operation.md) | 76,222 | ✅ |
| `agent-name` | [011_agent_name.md](../envelope/011_agent_name.md) | 22,415 | ❌ |
| `custom-title` | [013_custom_title.md](../envelope/013_custom_title.md) | 4,276 | ❌ |
| `pr-link` | [014_pr_link.md](../envelope/014_pr_link.md) | 677 | ✅ |
| `frame-link` | [019_frame_link.md](../envelope/019_frame_link.md) | 6 | ✅ |

### Field Contract

**Guaranteed present** on 100% of Class B lines:

| Field | Type | Role |
|-------|------|------|
| `sessionId` | string | Owning session — the only common field the whole class carries |

**Guaranteed present on three members only** — `queue-operation`, `pr-link`, `frame-link`, at 100% each:

| Field | Type | Role |
|-------|------|------|
| `timestamp` | string | ISO-8601 event time, orderable against Class A lines |

**Guaranteed absent** on 100% of Class B lines: `uuid`, `parentUuid`, `cwd`, `version`, `gitBranch`, `userType`, `isSidechain`.

Beyond these, each kind carries only its own payload — documented on its envelope instance.

### Correlation

- **By session** — `sessionId` attributes every Class B line without reference to the file it was found in. This is the class's one correlation capability and it is complete.
- **By time, for three members only** — `queue-operation`, `pr-link`, and `frame-link` can be ordered against Class A entries. The other six can be ordered only by position within the file.
- **Not by identity** — no `uuid` means nothing can reference a Class B line, and it references nothing. Class B lines are leaves.
- **Not by thread** — no `parentUuid` means Class B lines sit outside the conversation chain entirely. A thread walk will never traverse one.
- **Not by release** — no `version` means a Class B kind's lifecycle cannot be read from its own lines. The counts here are store-wide totals with no version attribution.

### Notes

**`timestamp` is the only field that splits the class**, and it does so without justifying a fourth class: the three kinds that carry it are otherwise field-identical to the six that do not. A consumer should test for `timestamp` rather than assume it from class membership.

**Six kinds are orderable only by file position.** For `last-prompt`, `mode`, `ai-title`, `permission-mode`, `agent-name`, and `custom-title`, the append-only file order is the sole temporal signal. Any operation that reorders or merges lines — re-sharding, concatenating, sorting by timestamp — destroys their sequence irrecoverably, and does so silently, since nothing in the line marks the loss.

**These are state transitions, not state.** `mode`, `permission-mode`, and `ai-title` each record a change; the value in force at any point is the most recent preceding line, not a per-turn annotation. Reading the newest line yields current state only if the consumer has already established the line is the newest.

**`queue-operation` is named in the storage invariant.** It is one of the four non-conversation types [`003_entry_type_format.md`](../../../../module/claude_storage/docs/invariant/003_entry_type_format.md) enumerates, and its fixture-versus-production `uuid` discrepancy is corrected in [readme.md](readme.md).

**This class is 17% of the store and carries no conversation content.** A consumer reconstructing a transcript can skip Class B in its entirety; a consumer auditing session configuration cannot, because permission and mode transitions live nowhere else.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| envelope_class | [readme.md](readme.md) | Class master index, presence matrix, and the `uuid` correction |
| envelope_class | [001_full_envelope.md](001_full_envelope.md) | Class A — all nine common fields |
| envelope_class | [003_detached.md](003_detached.md) | Class C — no common fields, correlation by handle |
| envelope | [`../envelope/readme.md`](../envelope/readme.md) | All 19 top-level kinds this class partitions |
| behavior | [`../behavior/004_b4_continue_flag.md`](../behavior/004_b4_continue_flag.md) | `--continue` behavior consuming `last-prompt` |
| behavior | [`../behavior/024_b24_from_pr.md`](../behavior/024_b24_from_pr.md) | `--from-pr` behavior producing `pr-link` |
| invariant | [`../../../../module/claude_storage/docs/invariant/003_entry_type_format.md`](../../../../module/claude_storage/docs/invariant/003_entry_type_format.md) | Skip-handling contract naming `queue-operation` |

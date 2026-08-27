# Envelope Class Doc Entity

### Scope

- **Purpose**: Define the structural classes that govern which common fields a session-log line is guaranteed to carry, and the correlation handles available when those fields are absent.
- **Responsibility**: Master file for the `envelope_class` collection — one instance per class, plus the measured field-presence matrix that partitions all 19 top-level kinds among them.
- **In Scope**: The three classes, their membership, the full presence matrix, the correlation handles each class offers, and the parsing consequences of each class boundary.
- **Out of Scope**: Per-kind payload semantics (→ [`../envelope/`](../envelope/readme.md), [`../attachment/`](../attachment/readme.md), [`../system_event/`](../system_event/readme.md)); conversation-entry field detail (→ [`../jsonl/001_common_fields.md`](../jsonl/001_common_fields.md)).

**Discriminator**: none. Class is not a field — it is a property of the `type` value, established by measurement.

### Overview Table

| ID | Name | Class | Kinds | Lines | Share | Responsibility |
|----|------|:-----:|------:|------:|------:|----------------|
| [001](001_full_envelope.md) | Full Envelope | A | 5 | 4,180,372 | 82.78% | All nine common fields present; the only class carrying `version` |
| [002](002_session_scoped.md) | Session-Scoped | B | 9 | 860,454 | 17.04% | `sessionId` plus payload; three members also carry `timestamp` |
| [003](003_detached.md) | Detached | C | 5 | 8,912 | 0.18% | No common fields at all; correlation via kind-specific handles |

Counts sum to 5,049,738 — every parsed line in the store.

### The Presence Matrix

The nine common fields, measured across all 19 top-level kinds. **Every value is exactly 100% or 0%.** No kind anywhere in the store shows partial presence of any common field, which is what makes a three-way class split the correct model rather than a per-kind field list.

| Kind | Class | `uuid` | `parentUuid` | `timestamp` | `sessionId` | `cwd` | `version` | `gitBranch` | `userType` | `isSidechain` |
|------|:-----:|:------:|:------------:|:-----------:|:-----------:|:-----:|:---------:|:-----------:|:----------:|:-------------:|
| [`assistant`](../envelope/001_assistant.md) | A | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| [`user`](../envelope/002_user.md) | A | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| [`attachment`](../envelope/003_attachment.md) | A | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| [`system`](../envelope/009_system.md) | A | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| [`progress`](../envelope/010_progress.md) | A | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| [`last-prompt`](../envelope/004_last_prompt.md) | B | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| [`mode`](../envelope/005_mode.md) | B | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| [`ai-title`](../envelope/006_ai_title.md) | B | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| [`permission-mode`](../envelope/007_permission_mode.md) | B | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| [`queue-operation`](../envelope/008_queue_operation.md) | B | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| [`agent-name`](../envelope/011_agent_name.md) | B | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| [`custom-title`](../envelope/013_custom_title.md) | B | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| [`pr-link`](../envelope/014_pr_link.md) | B | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| [`frame-link`](../envelope/019_frame_link.md) | B | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| [`file-history-snapshot`](../envelope/012_file_history_snapshot.md) | C | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| [`started`](../envelope/015_started.md) | C | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| [`result`](../envelope/016_result.md) | C | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| [`summary`](../envelope/017_summary.md) | C | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| [`fork-context-ref`](../envelope/018_fork_context_ref.md) | C | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |

`timestamp` is the only common field whose presence cuts *across* a class rather than aligning with it — three Class B kinds carry it and six do not. That intra-class split is documented in [002](002_session_scoped.md); it does not warrant a fourth class, because those three kinds are otherwise identical to their six siblings.

### Correction Supplied to the Storage Invariant

[`003_entry_type_format.md`](../../../../module/claude_storage/docs/invariant/003_entry_type_format.md) states, under *UUID field presence is not a type discriminator*, that `queue-operation` and `summary` "are confirmed to carry a `uuid` in the IN-3/IN-4 contract-test fixtures". The statement is accurate about the fixtures and inaccurate about production data:

| Kind | Fixture (IN-3/IN-4) | Full store | Independent re-check |
|------|---------------------|------------|----------------------|
| [`queue-operation`](../envelope/008_queue_operation.md) | carries `uuid` | 0 of 76,222 carry `uuid` | 0 of 21,104 |
| [`summary`](../envelope/017_summary.md) | carries `uuid` | 0 of 178 carry `uuid` | — (absent from re-check sample) |

Real Claude Code never writes `uuid` on either kind. The invariant's *behavioral* conclusions are unaffected — both kinds are skipped either way — and the document itself already flags that none of the four non-conversation types it names had a verified real-world artifact. This collection supplies that missing artifact and narrows the claim: the fixtures are unrepresentative on this field, so a `uuid`-presence assumption validated only against them does not hold in production.

**The parser rejects Class B and C at the `uuid` check, not the `type` check.** [`claude_storage_core`](../../../../module/claude_storage_core/src/entry.rs)'s `Entry::from_json_line` reads `uuid` before dispatching on `type`, so a Class B or C line fails there and never reaches type dispatch. The observable result — line skipped — is identical, but the recorded failure reason is "missing `uuid`" rather than "unrecognized type". Any diagnostic that counts skip reasons will attribute 869,366 lines to the wrong cause.

### Evidence Base

Every count, share, and presence rate in this collection derives from a full scan of the local session store:

| Property | Value |
|----------|-------|
| Session files scanned | 18,332 |
| Lines parsed | 5,049,738 |
| Unparseable lines | 37 (0.0007%) |
| Snapshot date | 2026-08-27 |
| Claude Code versions represented | 2.0.56 – 2.1.220 (20 distinct) |

Field presence comes from a second, independent full pass over the same store. The store is live and append-only, so absolute counts drift upward between passes; the presence/absence contract does not.

### Type-Specific Requirements

All `envelope_class` doc instances must include:

1. **Title**: `# ENVELOPE CLASS: {Class Name}` — using `ENVELOPE CLASS` as the type prefix
2. **Scope** (H3): 4 required bullets — Purpose, Responsibility, In Scope, Out of Scope
3. **Membership** (H3): the kinds in this class, with line counts and links to their envelope instances
4. **Field Contract** (H3): which of the nine common fields are guaranteed present and which are guaranteed absent
5. **Correlation** (H3): what a consumer can join this class's lines against, given the fields it has
6. **Notes** (H3): parsing consequences and stability caveats
7. **Cross-References** (H3): flat table with `Type | File | Responsibility` columns

### Stability Caveat

**The class split is a measurement, not a declared contract.** Claude Code publishes no schema for these lines. The 100%/0% cleanliness across 5,049,738 lines and 20 versions is strong evidence the split is intentional, but a future release may add a common field to a Class B kind without notice. Consumers should treat presence as optional and dispatch defensively rather than encoding these classes as hard assertions.

### Cross-Collection Dependencies

**This entity depends on**:
- [`../envelope/`](../envelope/readme.md) — the 19 top-level kinds this collection partitions

**This entity consumed by**:
- [`../../../../module/claude_storage_core/src/entry.rs`](../../../../module/claude_storage_core/src/entry.rs) — parser whose `uuid`-first check this collection explains
- [`../../../../module/claude_storage/docs/invariant/003_entry_type_format.md`](../../../../module/claude_storage/docs/invariant/003_entry_type_format.md) — skip-handling contract corrected above
- [`../jsonl/001_common_fields.md`](../jsonl/001_common_fields.md) — common-field schema, which this collection scopes to Class A

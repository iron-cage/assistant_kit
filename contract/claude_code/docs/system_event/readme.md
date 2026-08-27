# System Event Doc Entity

### Scope

- **Purpose**: Enumerate every `system.subtype` value — the third dispatch level of the session log, carrying lifecycle, telemetry, and error events.
- **Responsibility**: Master file for the `system_event` collection — one instance per subtype, with its fields, presence rates, and severity semantics.
- **In Scope**: All 10 observed `subtype` values; per-subtype field tables with types and presence rates; observed frequency; `level` and `isMeta` availability.
- **Out of Scope**: The `system` envelope itself (→ [`../envelope/009_system.md`](../envelope/009_system.md)); `attachment.type` payloads (→ [`../attachment/`](../attachment/readme.md)); the Class A field contract (→ [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md)).

**Discriminator**: `subtype`, on lines where the top-level `type` is `"system"`.

### Overview Table

| ID | Name | `subtype` | Lines | Share | `level` | `isMeta` | Responsibility |
|----|------|-----------|------:|------:|:-------:|:--------:|----------------|
| [001](001_compact_boundary.md) | Compact Boundary | `compact_boundary` | 18,282 | 40.45% | ✅ | 89% | Context compaction occurred, with the logical thread re-anchor |
| [002](002_local_command.md) | Local Command | `local_command` | 15,599 | 34.51% | ✅ | ✅ | A locally-executed slash command |
| [003](003_turn_duration.md) | Turn Duration | `turn_duration` | 8,058 | 17.83% | — | ✅ | Wall-clock duration and message count of a completed turn |
| [004](004_away_summary.md) | Away Summary | `away_summary` | 1,801 | 3.98% | — | ✅ | Summary generated while the user was away |
| [005](005_api_error.md) | API Error | `api_error` | 1,271 | 2.81% | ✅ | — | An API failure with complete retry accounting |
| [006](006_bridge_status.md) | Bridge Status | `bridge_status` | 86 | 0.190% | — | ✅ | Remote-control bridge connection status |
| [007](007_model_consent_fallback.md) | Model Consent Fallback | `model_consent_fallback` | 40 | 0.088% | ✅ | ✅ | Model unavailable — fallback offered and chosen |
| [008](008_scheduled_task_fire.md) | Scheduled Task Fire | `scheduled_task_fire` | 32 | 0.071% | — | ✅ | A scheduled task fired |
| [009](009_informational.md) | Informational | `informational` | 31 | 0.069% | ✅ | ✅ | General informational notice |
| [010](010_agents_killed.md) | Agents Killed | `agents_killed` | 1 | 0.0022% | — | ✅ | Running subagents were terminated |

Instances are numbered by descending observed frequency. Counts sum to exactly 45,201, matching the `system` envelope total in [`../envelope/009_system.md`](../envelope/009_system.md).

### Severity and Meta Fields

Neither `level` nor `isMeta` is universal on `system` lines, and their absences do not coincide:

- **`level`** is present on 5 of 10 subtypes, always at 100% when present. Its absence is not a default severity — it means the subtype carries no severity at all.
- **`isMeta`** is absent entirely on [`api_error`](005_api_error.md) and missing from 2,054 [`compact_boundary`](001_compact_boundary.md) lines. It is the one field in this collection with genuinely partial presence *within* a subtype rather than cleanly per-subtype.

### The Only Error Channel

[`api_error`](005_api_error.md) is the sole in-log record of API failure, and it is self-contained: `retryAttempt`, `maxRetries`, and `retryInMs` are all universal on it, so backoff behavior is fully reconstructable without external telemetry. At 1,271 occurrences, API failure is routine rather than exceptional — any consumer computing session success rates must account for it.

### The Thread-Repair Record

[`compact_boundary`](001_compact_boundary.md) carries `logicalParentUuid`, a pointer past the compaction gap. This is the documented exception to `parentUuid` self-containment: the raw chain breaks at a boundary, and this field restores continuity. A consumer walking the thread without it will see every compacted session as several disconnected fragments.

### Evidence Base

Every count, share, and presence rate in this collection derives from a full scan of the local session store:

| Property | Value |
|----------|-------|
| Session files scanned | 18,332 |
| Lines parsed | 5,049,738 |
| Unparseable lines | 37 (0.0007%) |
| Snapshot date | 2026-08-27 |
| Claude Code versions represented | 2.0.56 – 2.1.220 (20 distinct) |

Field types and presence rates come from a second, independent full pass over the same store. The store is live and append-only, so absolute counts drift upward between passes; ratios and the presence/absence contract do not.

**Store-range caveat**: the oldest data in this store is 2.0.56, so a kind observed across the full range has a `Since` floor of 2.0.56 — an artifact of the sample, not a claim about when the kind was introduced. Only a range starting or ending *strictly inside* 2.0.56 – 2.1.220 carries a real lifecycle signal.

### Type-Specific Requirements

All `system event` doc instances must include:

1. **Title**: `# SYSTEM EVENT: {Concept Name}` — using `SYSTEM EVENT` as the type prefix
2. **Scope** (H3): 4 required bullets — Purpose, Responsibility, In Scope, Out of Scope
3. **Schema** (H3): discriminator line, field table with type and presence rate, and a real captured JSON example
4. **Notes** (H3): parsing considerations, presence anomalies, and known exceptions
5. **Since** (H3): observed version range with the store-range caveat applied
6. **Cross-References** (H3): flat table with `Type | File | Responsibility` columns

### Parsing Considerations

- **Frequencies are workload-dependent.** [`scheduled_task_fire`](008_scheduled_task_fire.md) and [`bridge_status`](006_bridge_status.md) are absent entirely from stores that use neither scheduling nor remote control; their absence says nothing about a version's capabilities.
- **[`agents_killed`](010_agents_killed.md) is a single observation** across 5,049,738 lines. Its field set is provisional.
- **`subtype` is universal on `system` and only on `system`.** No other top-level kind carries it.

### Cross-Collection Dependencies

**This entity depends on**:
- [`../envelope/009_system.md`](../envelope/009_system.md) — the envelope carrying every subtype in this collection
- [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) — Class A common-field contract these lines satisfy

**This entity consumed by**:
- [`../fault/readme.md`](../fault/readme.md) — fault taxonomy for classifying `api_error`
- [`../model/readme.md`](../model/readme.md) — model catalog referenced by `model_consent_fallback`
- [`../behavior/017_b17_parentuuid_self_contained.md`](../behavior/017_b17_parentuuid_self_contained.md) — self-containment rule and its `compact_boundary` exception
- [`../behavior/025_b25_auto_compact_window.md`](../behavior/025_b25_auto_compact_window.md) — auto-compaction window driving `compact_boundary`

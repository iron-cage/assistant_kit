# Invariant: Schema Version

### Scope

- **Purpose**: Guarantee that every event record carries a schema version field enabling forward-compatible parsing.
- **Responsibility**: Define the `v` field contract, specify version-1 semantics, and describe how readers handle unknown versions.
- **In Scope**: `v` field presence in all emitted events, current version value (1), older-reader/newer-record compatibility rule.
- **Out of Scope**: Append-only constraint (→ `invariant/001_append_only.md`), crash durability (→ `invariant/002_crash_safety.md`).

### Invariant Statement

Every event record contains a `v` field (unsigned integer) identifying the schema version. The current version is `1`. This field enables forward-compatible parsing: older readers ignore unknown fields added in later versions; newer readers can apply version-specific parsing logic when encountering older records.

Version contract: a reader compiled against schema v1 can read any event where `v == 1`, regardless of extra fields present. Events with `v > 1` are parsed as raw JSON and returned with the `v` and `type` fields extracted but all other fields as `Option::None`.

### Measurement

- **Threshold**: 100% of events emitted by this version have `v == 1`
- **Method**: Unit test in `writer_test.rs` — serialize one event per type and assert the `"v":1` prefix is present in each serialized line

### Violation Consequences

- Emitting events without a `v` field prevents readers from applying version-specific parsing logic
- Bumping `v` without updating reader dispatch logic causes all new events to be parsed as raw/opaque records
- Using a non-integer `v` type breaks readers that rely on integer comparison for version dispatch

### Sources

| File | Relationship |
|------|--------------|
| `src/event.rs` | `EventRecord.v` field definition |
| `src/reader.rs` | Version-aware deserialization logic |

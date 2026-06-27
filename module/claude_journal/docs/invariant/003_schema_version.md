# Schema Version

**Status**: Planned | **Since**: 1.3.0

## Description

Every event record contains a `v` field (unsigned integer) identifying the schema version. The current version is `1`. This field enables forward-compatible parsing: older readers ignore unknown fields added in later versions; newer readers can apply version-specific parsing logic when encountering older records.

The version contract: a reader compiled against schema v1 can read any event where `v == 1`, regardless of extra fields present. Events with `v > 1` are parsed as raw JSON and returned with the `v` and `type` fields extracted but all other fields as `Option::None`.

## Measurement

- **Threshold**: 100% of events emitted by this version have `v == 1` (measured by structural test — emit all 8 event types, assert `v` field is `1` in each serialized line)
- **Method**: Unit test in `writer_test.rs` serializes one event per type and asserts the `"v":1` prefix

## Sources

- `src/event.rs` — `EventRecord.v` field
- `src/reader.rs` — version-aware deserialization

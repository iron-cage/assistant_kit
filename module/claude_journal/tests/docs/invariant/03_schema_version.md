# Test: Invariant — Schema Version

Test case planning for [invariant/003_schema_version.md](../../../docs/invariant/003_schema_version.md). Tests validate that every event emitted by `JournalWriter` has `v == 1` in both the deserialized struct and the raw JSONL bytes.

**Source:** [invariant/003_schema_version.md](../../../docs/invariant/003_schema_version.md)
**Related:** [feature/002_event_schema.md](../../../docs/feature/002_event_schema.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | All 8 event types have `v == 1` in deserialized `EventRecord.v` field | Struct Invariant |
| IN-2 | All 8 event types have `"v":1` in raw JSONL bytes as written by `JournalWriter` | Raw Bytes Invariant |
| IN-3 | `EventRecord::new()` always initializes `v` to `1` regardless of `EventType` | Constructor |

## Test Coverage Summary

- Struct Invariant: 1 test (IN-1)
- Raw Bytes Invariant: 1 test (IN-2)
- Constructor: 1 test (IN-3)

**Total:** 3 invariant test cases

## Architectural Constraint

IN-1 uses `JournalWriter::append()` + `JournalReader::query()` round-trip. IN-2 reads the raw file bytes after writing. IN-3 is a pure unit test of `EventRecord::new()`.

---

### IN-1: Deserialized events have `v == 1` for all 8 event types

- **Given:** one `EventRecord::new(et)` for each of the 8 `EventType` variants written via `JournalWriter`
- **When:** `JournalReader::query(default_filter)` reads them back
- **Then:** `events.len() == 8`; `ev.v == 1` for every event in the result
- **Source:** [invariant/003_schema_version.md](../../../docs/invariant/003_schema_version.md) Threshold: 100% of events have v==1

---

### IN-2: Raw JSONL bytes contain `"v":1` for all 8 event types

- **Given:** journal file written with one event per `EventType` variant via `JournalWriter`
- **When:** `std::fs::read_to_string(today_file)` and iterate non-empty lines
- **Then:** every line contains the substring `"v":1`; no line contains `"v":0` or `"v":2`
- **Source:** [invariant/003_schema_version.md](../../../docs/invariant/003_schema_version.md) Method: unit test asserts `"v":1` prefix

---

### IN-3: `EventRecord::new()` initializes `v` to `1`

- **Given:** calls to `EventRecord::new(et)` for all 8 `EventType` variants (no filesystem I/O)
- **When:** check `ev.v` field directly on the constructed struct
- **Then:** `ev.v == 1` for all 8 constructed records
- **Source:** [invariant/003_schema_version.md](../../../docs/invariant/003_schema_version.md) Rule: current version is 1

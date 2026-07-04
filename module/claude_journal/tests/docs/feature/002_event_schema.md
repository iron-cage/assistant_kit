# Test: Feature — Event Schema

### Scope

- **Purpose**: FT- test cases verifying `EventRecord` JSON serialization fields, types, and value formats.
- **Responsibility**: Acceptance criteria confirming required fields, schema version, event-type discriminators, timestamp format, and numeric/array field serialization are correct.
- **In Scope**: `v`/`ts`/`type` required fields, schema version value, 8 `EventType` discriminator strings, ISO 8601 timestamp format, `retry_class_counts` array shape, numeric field JSON types.
- **Out of Scope**: File creation and level-control behavior (-> `001_event_journaling.md`), rotation and pruning (-> `003_rotation.md`), schema-version invariant (-> `../invariant/003_schema_version.md`).

Test case planning for [feature/002_event_schema.md](../../../docs/feature/002_event_schema.md). Tests validate required fields, schema version, timestamp format, event type discriminators, array field structure, and numeric type serialization.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | Every event contains non-null `v`, `ts`, `type` fields (AC-001) | Required Fields |
| FT-2 | `v` field is `1` for all 8 event types (AC-002) | Schema Version |
| FT-3 | `type` field matches one of 8 defined EventType strings (AC-003) | Type Discriminator |
| FT-4 | `ts` field matches ISO 8601 UTC millisecond format `YYYY-MM-DDTHH:MM:SS.sssZ` (AC-004) | Timestamp Format |
| FT-5 | `retry_class_counts` serializes as a 6-element JSON array (AC-007) | Array Field |
| FT-6 | Numeric fields (`cost_usd`, `duration_ms`, `input_tokens`) serialize as JSON numbers, not strings (AC-008) | Numeric Types |

## Test Coverage Summary

- Required Fields: 1 test (FT-1)
- Schema Version: 1 test (FT-2)
- Type Discriminator: 1 test (FT-3)
- Timestamp Format: 1 test (FT-4)
- Array Field: 1 test (FT-5)
- Numeric Types: 1 test (FT-6)

**Total:** 6 tests

## Architectural Constraint

All tests serialize an `EventRecord` via `serde_json::to_string()` and then parse the resulting JSON value to verify field types and values. No filesystem I/O is required for FT-1 through FT-6 — these are unit-level serialization tests.

FT-3 verifies the 8 known type strings: `"execution"`, `"credential"`, `"gate_wait"`, `"retry"`, `"timeout"`, `"runner_retry"`, `"validation_retry"`, `"interactive"`.

---

### FT-1: `v`, `ts`, `type` are always present and non-null

- **Given:** `EventRecord::new(EventType::Execution)`
- **When:** `serde_json::to_string(&ev)`; parse as `serde_json::Value`
- **Then:** `value["v"]` is not `Value::Null`; `value["ts"]` is not `Value::Null`; `value["type"]` is not `Value::Null`
- **Source:** [feature/002_event_schema.md](../../../docs/feature/002_event_schema.md) AC-001

---

### FT-2: `v` field is `1` for all 8 event types

- **Given:** one `EventRecord` for each of the 8 `EventType` variants
- **When:** serialize each to JSON; parse `value["v"]`
- **Then:** `value["v"] == 1` (JSON number) for all 8 types
- **Source:** [feature/002_event_schema.md](../../../docs/feature/002_event_schema.md) AC-002

---

### FT-3: `type` field matches exact EventType string

- **Given:** `EventRecord::new(EventType::GateWait)` and one record per variant
- **When:** serialize; parse `value["type"]` as string
- **Then:** `EventType::GateWait` → `"gate_wait"`; `EventType::RunnerRetry` → `"runner_retry"`; `EventType::ValidationRetry` → `"validation_retry"`; all 8 values match the canonical lowercase snake_case strings
- **Source:** [feature/002_event_schema.md](../../../docs/feature/002_event_schema.md) AC-003

---

### FT-4: `ts` field is ISO 8601 UTC with millisecond precision

- **Given:** `EventRecord::new(EventType::Execution)` (timestamp generated at construction)
- **When:** serialize; parse `value["ts"]` as string `ts_str`
- **Then:** `ts_str.len() == 24` (e.g., `"2026-06-27T14:30:00.123Z"`); `ts_str.ends_with('Z')`; the date-time part before `Z` parses as a valid datetime
- **Source:** [feature/002_event_schema.md](../../../docs/feature/002_event_schema.md) AC-004

---

### FT-5: `retry_class_counts` serializes as 6-element array

- **Given:** `EventRecord` with `fields.retry_class_counts = Some([1,2,3,4,5,6])`
- **When:** serialize; parse `value["retry_class_counts"]`
- **Then:** the value is a JSON array of 6 numbers `[1,2,3,4,5,6]`; index 0 = Transient count, index 5 = Unknown count
- **Source:** [feature/002_event_schema.md](../../../docs/feature/002_event_schema.md) AC-007

---

### FT-6: Numeric fields serialize as JSON numbers

- **Given:** `EventRecord` with `fields.cost_usd = Some(0.001)`, `fields.duration_ms = Some(1234)`, `fields.input_tokens = Some(100)`
- **When:** serialize; parse the values
- **Then:** `value["cost_usd"]` is `Value::Number(0.001)` (not a string `"0.001"`); `value["duration_ms"]` is `Value::Number(1234)`; `value["input_tokens"]` is `Value::Number(100)`
- **Source:** [feature/002_event_schema.md](../../../docs/feature/002_event_schema.md) AC-008

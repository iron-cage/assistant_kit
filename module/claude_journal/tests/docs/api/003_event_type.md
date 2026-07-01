# Test: API — EventType, EventRecord, EventFields

Test case planning for [api/003_event_type.md](../../../docs/api/003_event_type.md). Tests validate `EventType::as_str()` correctness, `from_str()` round-trip and unknown-type handling, `EventFields::default()`, `EventRecord::new()` initialization, and case-sensitivity.

**Source:** [api/003_event_type.md](../../../docs/api/003_event_type.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| AP-1 | `EventType::as_str()` returns exact lowercase snake_case for all 8 variants | Serialization |
| AP-2 | `EventType::from_str()` round-trips with `as_str()` for all 8 variants | Round-Trip |
| AP-3 | `EventType::from_str("bogus")` returns `None` | Unknown Type |
| AP-4 | `EventType::from_str()` is case-sensitive: `"Execution"` returns `None` | Case Sensitivity |
| AP-5 | `EventFields::default()` returns all fields as `None` | Default Fields |
| AP-6 | `EventRecord::new(et)` sets `v = 1`, valid UTC `ts`, and `event_type == et` | Record Constructor |

## Test Coverage Summary

- Serialization: 1 test (AP-1)
- Round-Trip: 1 test (AP-2)
- Unknown Type: 1 test (AP-3)
- Case Sensitivity: 1 test (AP-4)
- Default Fields: 1 test (AP-5)
- Record Constructor: 1 test (AP-6)

**Total:** 6 tests

---

### AP-1: `as_str()` returns canonical strings for all 8 variants

- **Given:** all 8 `EventType` variants
- **When:** `event_type.as_str()` for each
- **Then:** exact strings: `"execution"`, `"credential"`, `"gate_wait"`, `"retry"`, `"timeout"`, `"runner_retry"`, `"validation_retry"`, `"interactive"`
- **Source:** [api/003_event_type.md](../../../docs/api/003_event_type.md) Behavioral Contract: `as_str()` mapping

---

### AP-2: `from_str()` round-trips with `as_str()`

- **Given:** all 8 `EventType` variants
- **When:** `EventType::from_str(et.as_str())` for each
- **Then:** returns `Some(et)` — exact same variant recovered
- **Source:** [api/003_event_type.md](../../../docs/api/003_event_type.md) Behavioral Contract: round-trip

---

### AP-3: `from_str("bogus")` returns `None`

- **Given:** unknown type strings: `"bogus"`, `""`, `"EXECUTION"`, `"gate-wait"`
- **When:** `EventType::from_str(s)` for each
- **Then:** all return `None`
- **Source:** [api/003_event_type.md](../../../docs/api/003_event_type.md) Behavioral Contract: returns `None` for unknown types

---

### AP-4: `from_str()` is case-sensitive

- **Given:** capitalized variants: `"Execution"`, `"GateWait"`, `"RETRY"`
- **When:** `EventType::from_str(s)` for each
- **Then:** all return `None` — only exact lowercase snake_case matches succeed
- **Source:** [api/003_event_type.md](../../../docs/api/003_event_type.md) Behavioral Contract: case-sensitive

---

### AP-5: `EventFields::default()` has all fields as `None`

- **Given:** `EventFields::default()`
- **When:** inspect all 33 fields
- **Then:** every `Option<_>` field is `None`; `retry_class_counts` is `None` (not `Some([0,0,0,0,0,0])`)
- **Source:** [api/003_event_type.md](../../../docs/api/003_event_type.md) Behavioral Contract: `EventFields::default()` is all-None

---

### AP-6: `EventRecord::new(et)` correct initialization

- **Given:** `EventRecord::new(EventType::Retry)`
- **When:** inspect `ev.v`, `ev.ts`, `ev.event_type`, `ev.fields`
- **Then:** `ev.v == 1`; `ev.event_type == EventType::Retry`; `ev.ts` matches `YYYY-MM-DDTHH:MM:SS.sssZ` pattern; `ev.fields` is all-None (default)
- **Source:** [api/003_event_type.md](../../../docs/api/003_event_type.md) Interface: `EventRecord::new()`

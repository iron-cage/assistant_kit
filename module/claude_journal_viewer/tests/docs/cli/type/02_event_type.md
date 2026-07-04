# Type :: `EventType`

Validation tests for the `EventType` enum. Tests validate case-insensitive
matching across the 8 canonical variants and invalid-variant error handling.

**Source:** [type/02_event_type.md](../../../../docs/cli/type/02_event_type.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `Execution`, `EXECUTION`, `execution` -> all match the same variant | Case Insensitivity |
| TC-2 | `gate_wait` -> valid variant accepted | Parsing |
| TC-3 | Invalid variant -> exit 1 listing valid options | Error Handling |

## Test Coverage Summary

- Case Insensitivity: 1 test (TC-1)
- Parsing: 1 test (TC-2)
- Error Handling: 1 test (TC-3)

**Total:** 3 test cases

## Test Cases

---

### TC-1: `Execution`, `EXECUTION`, `execution` -> all match the same variant

- **Given:** journal containing `execution`-type events
- **When:** `clj .list type::Execution`, `clj .list type::EXECUTION`, `clj .list type::execution`
- **Then:** all three produce identical results, matching the `execution` variant
- **Exit:** 0 for all three
- **Source:** [type/02_event_type.md](../../../../docs/cli/type/02_event_type.md)

---

### TC-2: `gate_wait` -> valid variant accepted

- **Given:** journal containing `gate_wait`-type events
- **When:** `clj .list type::gate_wait`
- **Then:** exit 0; only `gate_wait` events are shown
- **Exit:** 0
- **Source:** [type/02_event_type.md](../../../../docs/cli/type/02_event_type.md)

---

### TC-3: Invalid variant -> exit 1 listing valid options

- **Given:** clean environment
- **When:** `clj .list type::not_a_real_type`
- **Then:** exit 1; stderr lists all 8 valid `EventType` variants
- **Exit:** 1
- **Source:** [type/02_event_type.md](../../../../docs/cli/type/02_event_type.md)

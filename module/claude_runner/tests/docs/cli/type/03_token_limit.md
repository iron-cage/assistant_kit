# Type :: `TokenLimit`

Validation tests for the `TokenLimit` semantic type (u32, 0–4294967295). Tests validate boundary values, overflow, and non-numeric rejection.

**Source:** [type.md](../../../../docs/cli/type.md#type--3-tokenlimit)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `0` → accepted (minimum) | Boundary |
| TC-2 | `4294967295` → accepted (maximum u32) | Boundary |
| TC-3 | `4294967296` → exit 1 (u32 overflow) | Boundary |
| TC-4 | `-1` → exit 1 (negative rejected) | Invalid Input |
| TC-5 | `abc` → exit 1 (non-numeric rejected) | Type Validation |
| TC-6 | Default `200000` applied when unset | Default |

## Test Coverage Summary

- Boundary: 3 tests (TC-1, TC-2, TC-3)
- Invalid Input: 1 test (TC-4)
- Type Validation: 1 test (TC-5)
- Default: 1 test (TC-6)

**Total:** 6 test cases

## Test Cases

---

### TC-1: `0` → accepted (minimum)

- **Given:** clean environment
- **When:** `clr --dry-run "test" --max-tokens 0`
- **Then:** Exit 0; assembled command contains `0` for max tokens
- **Exit:** 0
- **Source:** [type.md — TokenLimit](../../../../docs/cli/type.md#type--3-tokenlimit)

---

### TC-2: `4294967295` → accepted (u32 max)

- **Given:** clean environment
- **When:** `clr --dry-run "test" --max-tokens 4294967295`
- **Then:** Exit 0; maximum u32 value accepted
- **Exit:** 0
- **Source:** [type.md — TokenLimit](../../../../docs/cli/type.md#type--3-tokenlimit)

---

### TC-3: `4294967296` → exit 1 (overflow)

- **Given:** clean environment
- **When:** `clr "test" --max-tokens 4294967296`
- **Then:** Exit 1; error indicating invalid `--max-tokens` value; u32 overflow rejected
- **Exit:** 1
- **Source:** [type.md — TokenLimit](../../../../docs/cli/type.md#type--3-tokenlimit)

---

### TC-4: `-1` → exit 1 (negative)

- **Given:** clean environment
- **When:** `clr "test" --max-tokens -1`
- **Then:** Exit 1; negative value rejected with error message
- **Exit:** 1
- **Source:** [type.md — TokenLimit](../../../../docs/cli/type.md#type--3-tokenlimit)

---

### TC-5: `abc` → exit 1 (non-numeric)

- **Given:** clean environment
- **When:** `clr "test" --max-tokens abc`
- **Then:** Exit 1; error: `"invalid --max-tokens value: abc\nExpected unsigned integer 0–4294967295"`
- **Exit:** 1
- **Source:** [type.md — TokenLimit](../../../../docs/cli/type.md#type--3-tokenlimit)

---

### TC-6: Default 200000 applied when unset

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command or env shows `200000` as the token limit default
- **Exit:** 0
- **Source:** [type.md — TokenLimit](../../../../docs/cli/type.md#type--3-tokenlimit)

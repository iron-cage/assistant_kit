# Test: `.settings.get`

Integration test planning for the `.settings.get` command. See [commands.md](../../../../docs/cli/commands.md) for specification.

## Test Factor Analysis

### Factor 1: `key::` (String, required)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Parameter not provided | Invalid: exit 1 |
| non-empty existing key | Key found in settings | Happy path |
| non-empty missing key | Key not in settings | Failure: exit 2 |
| empty string | Key cannot be empty | Invalid: exit 1 |

### Factor 2: `v::` / verbosity (Integer, optional, default 1)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default value 1, `key: value` | Default behavior |
| 0 | Bare value only | Minimum output |
| 1 | `key: value` labeled | Nominal |
| 3 | Out-of-range integer | Invalid: exit 1 |

### Factor 3: `format::` (String, optional, default "text")

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default text output | Default behavior |
| `text` | Explicit text output | Explicit valid |
| `json` | `{"key":"..","value":..}` | Alternate valid |
| `xml` | Unrecognized value | Invalid: exit 1 |

### Factor 4: settings.json state (State)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| valid with key | Key present | Happy path |
| valid without key | Key absent | Failure: exit 2 |
| missing | File does not exist | Failure: exit 2 |

### Factor 5: Unknown parameters

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| none | No unknown params | Happy path |
| present | e.g. `bogus::x` | Invalid: exit 1 |

---

## Test Matrix

### Positive Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-2 | `key::existing` → value returned, exit 0 | P | 0 | F1=existing | [read_commands_test.rs] |
| IT-4 | `v::0` → bare value only | P | 0 | F1=existing, F2=0 | [read_commands_test.rs] |
| TC-180 | `v::1` → `key: value` labeled | P | 0 | F1=existing, F2=1 | [read_commands_test.rs] |
| TC-182 | `format::json` → `{"key":"..","value":".."}` | P | 0 | F1=existing, F3=json | [read_commands_test.rs] |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-1 | No `key::` → exit 1, required parameter | N | 1 | F1=absent | [read_commands_test.rs] |
| IT-3 | `key::nonexistent` → exit 2, key not found | N | 2 | F1=missing | [read_commands_test.rs] |
| TC-184 | File missing → exit 2 | N | 2 | F4=missing | [read_commands_test.rs] |
| TC-237 | Without `key::` → error mentions `key::` | N | 1 | F1=absent | [read_commands_test.rs] |
| IT-5 | `key::` (empty value) → exit 1 | N | 1 | F1=empty | new |
| IT-6 | `format::xml` → exit 1 | N | 1 | F3=xml | new |
| IT-7 | `bogus::x` → exit 1 | N | 1 | F5=present | new |
| IT-8 | `v::3` → exit 1, out of range | N | 1 | F2=3 | new |

### Summary

- **Total:** 12 tests (4 positive, 8 negative)
- **Negative ratio:** 66.7% ✅ (≥40%)
- **TC range:** IT-1 to IT-8

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Key found and returned | IT-2, IT-4, TC-180, TC-182 |
| 1 | Invalid arguments | IT-1, TC-237, IT-5, IT-6, IT-7, IT-8 |
| 2 | Key not found or file missing | IT-3, TC-184 |

### Required Parameter Enforcement (FR-04)

`key::` is semantically required. Absence → exit 1 "key is required" (IT-1, TC-237).
Empty value → exit 1 "key value cannot be empty" (IT-5).

### v::0 Output Format

IT-4 verifies exact bare-value format: `text.trim() == "thevalue"` (no label, no newline).
This is critical for script consumption.

---

## Test Case Details

---

### IT-1: No `key::` → exit 1

- **Given:** `HOME=<tmp>` with valid settings.
- **When:**
  `cm .settings.get`
  **Expected:** Exit 1; stderr mentions `key::`.
- **Then:** descriptive error
- **Exit:** 1

---

### IT-2: Existing key → value, exit 0

- **Given:** `HOME=<tmp>`; settings has `myKey = "myValue"`.
- **When:**
  `cm .settings.get key::myKey`
  **Expected:** Exit 0; output contains "myValue".
- **Then:** value shown
- **Exit:** 0

---

### IT-3: Nonexistent key → exit 2

- **Given:** `HOME=<tmp>`; settings has different key.
- **When:**
  `cm .settings.get key::nosuchkey`
  **Expected:** Exit 2.
- **Then:** see spec
- **Exit:** 2

---

### IT-4: `v::0` → bare value only

- **Given:** `HOME=<tmp>`; settings has `k = "thevalue"`.
- **When:**
  `cm .settings.get key::k v::0`
  **Expected:** Exit 0; stdout trimmed equals "thevalue" exactly.
- **Then:** exact bare value
- **Exit:** 0

---

### IT-5: `key::` (empty value) → exit 1

- **Given:** clean environment
- **When:**
  `cm .settings.get key::`
  **Expected:** Exit 1; stderr mentions empty key.
- **Then:** see spec
- **Exit:** 1

---

### IT-6: `format::xml` → exit 1

- **Given:** clean environment
- **When:**
  `cm .settings.get format::xml key::foo`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-7: `bogus::x` → exit 1

- **Given:** clean environment
- **When:**
  `cm .settings.get bogus::x`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-8: `v::3` → exit 1

- **Given:** clean environment
- **When:**
  `cm .settings.get v::3 key::foo`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

# Test: `.settings.get`

### Scope

- **Purpose**: Integration test cases for the `.settings.get` command.
- **Responsibility**: Test factor analysis, case index, and expected behavior for single-key retrieval.
- **In Scope**: Key lookup, missing key, verbosity levels, output formats, file state.
- **Out of Scope**: Parameter edge cases (→ `../param/`), group interactions (→ `../param_group/`).

Integration test planning for the `.settings.get` command. See [command/readme.md](../../../../docs/cli/command/readme.md) for specification.

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
| IT-2 | `key::existing` → value returned, exit 0 | P | 0 | F1=existing | [read_settings_test.rs] |
| IT-4 | `v::0` → bare value only | P | 0 | F1=existing, F2=0 | [read_settings_test.rs] |
| IT-9 | `v::1` → `key: value` labeled | P | 0 | F1=existing, F2=1 | [read_settings_test.rs] |
| IT-10 | `format::json` → `{"key":"..","value":".."}` | P | 0 | F1=existing, F3=json | [read_settings_test.rs] |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-1 | No `key::` → exit 1, required parameter | N | 1 | F1=absent | [read_settings_test.rs] |
| IT-3 | `key::nonexistent` → exit 2, key not found | N | 2 | F1=missing | [read_settings_test.rs] |
| IT-11 | File missing → exit 2 | N | 2 | F4=missing | [read_settings_test.rs] |
| IT-12 | Without `key::` → error mentions `key::` | N | 1 | F1=absent | [read_settings_test.rs] |
| IT-5 | `key::` (empty value) → exit 1 | N | 1 | F1=empty | new |
| IT-6 | `format::xml` → exit 1 | N | 1 | F3=xml | new |
| IT-7 | `bogus::x` → exit 1 | N | 1 | F5=present | new |
| IT-8 | `v::3` → exit 1, out of range | N | 1 | F2=3 | new |

### Summary

- **Total:** 12 tests (4 positive, 8 negative)
- **Negative ratio:** 66.7% ✅ (≥40%)
- **IT range:** IT-1 to IT-12

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Key found and returned | IT-2, IT-4, IT-9, IT-10 |
| 1 | Invalid arguments | IT-1, IT-5, IT-6, IT-7, IT-8, IT-12 |
| 2 | Key not found or file missing | IT-3, IT-11 |

### Required Parameter Enforcement (FR-04)

`key::` is semantically required. Absence → exit 1 "key is required" (IT-1, IT-12).
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
  `clv .settings.get`
- **Expected:** Exit 1; stderr mentions `key::`.
- **Then:** descriptive error
- **Exit:** 1

---

### IT-2: Existing key → value, exit 0

- **Given:** `HOME=<tmp>`; settings has `myKey = "myValue"`.
- **When:**
  `clv .settings.get key::myKey`
- **Expected:** Exit 0; output contains "myValue".
- **Then:** value shown
- **Exit:** 0

---

### IT-3: Nonexistent key → exit 2

- **Given:** `HOME=<tmp>`; settings has different key.
- **When:**
  `clv .settings.get key::nosuchkey`
- **Expected:** Exit 2.
- **Then:** see spec
- **Exit:** 2

---

### IT-4: `v::0` → bare value only

- **Given:** `HOME=<tmp>`; settings has `k = "thevalue"`.
- **When:**
  `clv .settings.get key::k v::0`
- **Expected:** Exit 0; stdout trimmed equals "thevalue" exactly.
- **Then:** exact bare value
- **Exit:** 0

---

### IT-5: `key::` (empty value) → exit 1

- **Given:** clean environment
- **When:**
  `clv .settings.get key::`
- **Expected:** Exit 1; stderr mentions empty key.
- **Then:** see spec
- **Exit:** 1

---

### IT-6: `format::xml` → exit 1

- **Given:** clean environment
- **When:**
  `clv .settings.get format::xml key::foo`
- **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-7: `bogus::x` → exit 1

- **Given:** clean environment
- **When:**
  `clv .settings.get bogus::x`
- **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-8: `v::3` → exit 1

- **Given:** clean environment
- **When:**
  `clv .settings.get v::3 key::foo`
- **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-9: `v::1` → `key: value` labeled

- **Given:** `HOME=<tmp>`; settings.json contains `myKey = "myValue"`
- **When:** `clv .settings.get key::myKey v::1`
- **Then:** exit 0; stdout is in `key: value` format (e.g., `myKey: myValue`)
- **Exit:** 0
- **Source:** [command/settings.md](../../../../docs/cli/command/settings.md)

---

### IT-10: `format::json` → `{"key":"..","value":".."}` object

- **Given:** `HOME=<tmp>`; settings.json contains `myKey = "myValue"`
- **When:** `clv .settings.get key::myKey format::json`
- **Then:** exit 0; stdout is a valid JSON object with `"key"` and `"value"` fields
- **Exit:** 0
- **Source:** [command/settings.md](../../../../docs/cli/command/settings.md)

---

### IT-11: File missing → exit 2

- **Given:** `HOME=<tmp>` with no `.claude/settings.json`
- **When:** `clv .settings.get key::anyKey`
- **Then:** exit 2; error references missing file or settings path
- **Exit:** 2
- **Source:** [command/settings.md](../../../../docs/cli/command/settings.md)

---

### IT-12: Without `key::` → error mentions `key::`

- **Given:** `HOME=<tmp>` with valid settings.json
- **When:** `clv .settings.get`
- **Then:** exit 1; error message contains the string `key::`
- **Exit:** 1
- **Source:** [command/settings.md](../../../../docs/cli/command/settings.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc174_settings_get_no_key_exits_1` | `tests/cli/read_settings_test.rs` |
| `tc176_settings_get_existing_key` | `tests/cli/read_settings_test.rs` |
| `tc177_settings_get_missing_key_exits_2` | `tests/cli/read_settings_test.rs` |
| `tc179_settings_get_v0_bare_value` | `tests/cli/read_settings_test.rs` |
| `tc180_settings_get_v1_labeled` | `tests/cli/read_settings_test.rs` |
| `tc182_settings_get_format_json` | `tests/cli/read_settings_test.rs` |
| `tc184_settings_get_file_missing_exits_2` | `tests/cli/read_settings_test.rs` |
| `tc237_settings_get_missing_key_error_format` | `tests/cli/read_settings_test.rs` |
| `tc490_settings_get_json_bool_unquoted` | `tests/cli/read_settings_test.rs` |
| `tc491_settings_get_json_number_unquoted` | `tests/cli/read_settings_test.rs` |
| `tc492_settings_get_json_string_quoted` | `tests/cli/read_settings_test.rs` |
| `tc505_settings_get_missing_key_error_contains_key` | `tests/cli/error_messages_test.rs` |
| `tc511_settings_get_absent_key_error_mentions_key` | `tests/cli/error_messages_test.rs` |

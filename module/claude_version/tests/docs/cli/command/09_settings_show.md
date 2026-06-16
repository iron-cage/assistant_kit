# Test: `.settings.show`

### Scope

- **Purpose**: Integration test cases for the `.settings.show` command.
- **Responsibility**: Test factor analysis, case index, and expected behavior for settings display.
- **In Scope**: Key-value output, verbosity levels, output formats, missing file handling.
- **Out of Scope**: Parameter edge cases (→ `../param/`), group interactions (→ `../param_group/`).

Integration test planning for the `.settings.show` command. See [command/readme.md](../../../../docs/cli/command/readme.md) for specification.

## Test Factor Analysis

### Factor 1: `v::` / verbosity (Integer, optional, default 1)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default value 1, labeled `key: value` | Default behavior |
| 0 | `key=value` compact format | Minimum output |
| 1 | `key: value` labeled | Nominal |
| 3 | Out-of-range integer | Invalid: exit 1 |

### Factor 2: `format::` (String, optional, default "text")

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default text output | Default behavior |
| `text` | Explicit text output | Explicit valid |
| `json` | JSON object mirroring settings file | Alternate valid |
| `xml` | Unrecognized value | Invalid: exit 1 |

### Factor 3: settings.json state (State)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| missing | File does not exist | Failure: exit 2 |
| empty `{}` | Valid but no keys | Empty state: exit 0 |
| valid with keys | Normal data | Happy path |
| malformed | Invalid JSON | Failure: exit 2 |

### Factor 4: HOME environment (Environmental)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| set | Normal path construction | Happy path |
| empty | Cannot resolve path | Failure: exit 2 |

### Factor 5: Type preservation in JSON (Content)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| string values | Shown quoted in text, JSON string | Nominal |
| boolean values | Shown as `true`/`false` in JSON | Type-preserving |
| integer values | Shown as number in JSON | Type-preserving |
| nested objects | Preserved verbatim in JSON | Structural fidelity |

### Factor 6: Unknown parameters

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| none | No unknown params | Happy path |
| present | e.g. `bogus::x` | Invalid: exit 1 |

---

## Test Matrix

### Positive Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-2 | Empty `{}` → empty output, exit 0 | P | 0 | F3=empty | [read_commands_test.rs] |
| IT-3 | Valid settings → keys shown, exit 0 | P | 0 | F3=valid | [read_commands_test.rs] |
| IT-9 | `v::0` → `key=value` format | P | 0 | F1=0, F3=valid | [read_commands_test.rs] |
| IT-10 | `format::json` → valid JSON object | P | 0 | F2=json, F3=valid | [read_commands_test.rs] |
| IT-4 | `format::json` preserves bool/number types | P | 0 | F2=json, F5=boolean | [read_commands_test.rs] |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-1 | File missing → exit 2 | N | 2 | F3=missing | [read_commands_test.rs] |
| IT-11 | Malformed JSON → exit 2 | N | 2 | F3=malformed | [read_commands_test.rs] |
| IT-12 | HOME not set → exit 2 | N | 2 | F4=empty | [read_commands_test.rs] |
| IT-5 | `bogus::x` → exit 1 | N | 1 | F6=present | new |
| IT-6 | `format::xml` → exit 1 | N | 1 | F2=xml | new |
| IT-7 | `v::3` → exit 1, out of range | N | 1 | F1=3 | new |
| IT-8 | Output goes to stdout only; stderr is empty | P | 0 | F3=valid, F4=set | new |

### Summary

- **Total:** 12 tests (6 positive, 6 negative)
- **Negative ratio:** 50.0% ✅ (≥40%)
- **IT range:** IT-1 to IT-12

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success | IT-2, IT-3, IT-4, IT-9, IT-10 |
| 1 | Invalid arguments | IT-5, IT-6, IT-7 |
| 2 | Runtime error (missing file, bad JSON, no HOME) | IT-1, IT-11, IT-12 |

### Settings File State Coverage

| State | Tests |
|-------|-------|
| Missing | IT-1 (exit 2) |
| Empty `{}` | IT-2 (exit 0, empty output) |
| Valid with keys | IT-3, IT-9, IT-10 |
| Malformed | IT-11 (exit 2) |

---

## Test Case Details

---

### IT-1: File missing → exit 2

- **Given:** `HOME=<tmp>` with no `.claude/settings.json`.
- **When:**
  `clv .settings.show`
- **Expected:** Exit 2.
- **Then:** see spec
- **Exit:** 2

---

### IT-2: Empty `{}` → empty output

- **Given:** `HOME=<tmp>`; `settings.json` = `{}`.
- **When:**
  `clv .settings.show`
- **Expected:** Exit 0; stdout is empty.
- **Then:** no output
- **Exit:** 0

---

### IT-3: Valid settings → keys shown

- **Given:** `HOME=<tmp>`; `settings.json` has `myKey = "myValue"`.
- **When:**
  `clv .settings.show`
- **Expected:** Exit 0; output contains "myKey" and "myValue".
- **Then:** key/value visible
- **Exit:** 0

---

### IT-4: `format::json` preserves bool/number types

- **Given:** `HOME=<tmp>`; settings has boolean and integer values.
- **When:**
  `clv .settings.show format::json`
- **Expected:** Exit 0; JSON output has `true`/`false` booleans and numeric integers (not quoted strings).
- **Then:** type preservation
- **Exit:** 0

---

### IT-5: `bogus::x` → exit 1

- **Given:** clean environment
- **When:**
  `clv .settings.show bogus::x`
- **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-6: `format::xml` → exit 1

- **Given:** clean environment
- **When:**
  `clv .settings.show format::xml`
- **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-7: `v::3` → exit 1

- **Given:** clean environment
- **When:**
  `clv .settings.show v::3`
- **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-8: Output goes to stdout only; stderr is empty

- **Given:** `HOME=<tmp>` with valid settings.json containing at least one key
- **When:** `clv .settings.show`
- **Then:** stdout is non-empty; stderr is empty
- **Exit:** 0
- **Source:** [command/readme.md](../../../../docs/cli/command/readme.md)

---

### IT-9: `v::0` → `key=value` format

- **Given:** `HOME=<tmp>`; settings.json has at least one key
- **When:** `clv .settings.show v::0`
- **Then:** exit 0; each output line is in compact `key=value` format (no spaces around `=`, no label prefix)
- **Exit:** 0
- **Source:** [command/settings.md](../../../../docs/cli/command/settings.md)

---

### IT-10: `format::json` → valid JSON object

- **Given:** `HOME=<tmp>`; settings.json has at least one key
- **When:** `clv .settings.show format::json`
- **Then:** exit 0; stdout is valid JSON object; top-level is `{}`; keys match settings file keys
- **Exit:** 0
- **Source:** [command/settings.md](../../../../docs/cli/command/settings.md)

---

### IT-11: Malformed JSON → exit 2

- **Given:** `HOME=<tmp>`; settings.json contains invalid JSON
- **When:** `clv .settings.show`
- **Then:** exit 2; error message references parse failure or invalid settings file
- **Exit:** 2
- **Source:** [command/settings.md](../../../../docs/cli/command/settings.md)

---

### IT-12: HOME not set → exit 2

- **Given:** HOME environment variable is unset
- **When:** `clv .settings.show`
- **Then:** exit 2; error message references HOME or settings path resolution failure
- **Exit:** 2
- **Source:** [command/settings.md](../../../../docs/cli/command/settings.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc161_settings_show_file_missing_exits_2` | `integration/read_commands_test.rs` |
| `tc162_settings_show_empty_file_exits_0` | `integration/read_commands_test.rs` |
| `tc163_settings_show_valid_file` | `integration/read_commands_test.rs` |
| `tc164_settings_show_v0_key_equals_value` | `integration/read_commands_test.rs` |
| `tc167_settings_show_format_json` | `integration/read_commands_test.rs` |
| `tc170_settings_show_malformed_file_exits_2` | `integration/read_commands_test.rs` |
| `tc171_settings_show_no_home_exits_2` | `integration/read_commands_test.rs` |
| `tc241_settings_show_json_preserves_types` | `integration/read_commands_test.rs` |
| `tc507_settings_show_no_home_error_mentions_home` | `integration/error_messages_test.rs` |
| `tc508_settings_show_home_unset_error_mentions_home` | `integration/error_messages_test.rs` |

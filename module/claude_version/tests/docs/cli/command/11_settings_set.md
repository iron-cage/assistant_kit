# Test: `.settings.set`

Integration test planning for the `.settings.set` command. See [command/readme.md](../../../../docs/cli/command/readme.md) for specification.

### Scope

- **Purpose**: Integration test cases for the `.settings.set` command.
- **Responsibility**: Test factor analysis, test case index, and expected behavior for `.settings.set`.
- **In Scope**: Command-level integration tests, exit codes, output verification.
- **Out of Scope**: Parameter edge cases (→ `../param/`), group interactions (→ `../param_group/`).

## Test Factor Analysis

### Factor 1: `key::` (String, required)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Parameter not provided | Invalid: exit 1 |
| non-empty | Valid key name | Happy path |
| empty string | Key cannot be empty | Invalid: exit 1 |

### Factor 2: `value::` (String, required)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent (with key present) | Parameter not provided | Invalid: exit 1 |
| `"true"` / `"false"` | Boolean literal strings | JSON bool |
| string parseable as i64 | Integer-like strings | JSON number |
| float-like string | Decimal string | JSON float |
| arbitrary string | Other strings | JSON string |
| empty string `""` | Valid — stores empty string | JSON string `""` |
| NaN / infinity strings | Float NaN/inf | JSON string (not number) |

### Factor 3: `dry::` (Boolean, optional, default 0)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default: real write | Default behavior |
| 1 | Preview only: no file change | Explicit true |
| 2 | Out-of-range boolean | Invalid: exit 1 |

### Factor 4: File state (State)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| missing | File does not exist | Created on write |
| existing with key | Key present | Overwrite existing |
| existing without key | Key absent | Append new key |

### Factor 5: HOME environment (Environmental)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| set | Normal path construction | Happy path |
| empty | Cannot resolve path | Failure: exit 2 |

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
| IT-1 | `value::true` → stores boolean `true` | P | 0 | F1=set, F2=true | [mutation_settings_set_test.rs] |
| IT-9 | `value::false` → stores boolean `false` | P | 0 | F1=set, F2=false | [mutation_settings_set_test.rs] |
| IT-2 | `value::0` → stores number `0` (NOT boolean) | P | 0 | F1=set, F2=0 | [mutation_settings_set_test.rs] |
| IT-10 | `value::42` → stores integer `42` | P | 0 | F1=set, F2=int | [mutation_settings_set_test.rs] |
| IT-11 | `value::hello` → stores quoted `"hello"` | P | 0 | F1=set, F2=string | [mutation_settings_set_test.rs] |
| IT-13 | Creates file when settings.json absent | P | 0 | F4=missing | [mutation_settings_set_test.rs] |
| IT-14 | Updates existing key without duplication | P | 0 | F4=existing-with-key | [mutation_settings_set_test.rs] |
| IT-3 | `dry::1` → shows preview, no file change | P | 0 | F3=1 | [mutation_settings_set_test.rs] |
| IT-15 | Adds new key to existing file | P | 0 | F4=existing-without-key | [mutation_settings_set_test.rs] |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-16 | Without `key::` → error mentions `key::` | N | 1 | F1=absent | [read_settings_test.rs] |
| IT-17 | `key::foo` without `value::` → error mentions `value::` | N | 1 | F2=absent | [read_settings_test.rs] |
| IT-18 | No `key::` → exit 1 | N | 1 | F1=absent | [mutation_settings_set_test.rs] |
| IT-19 | `key::` present but no `value::` → exit 1 | N | 1 | F1=set, F2=absent | [mutation_settings_set_test.rs] |
| IT-20 | HOME not set → exit 2 | N | 2 | F5=empty | [mutation_settings_set_test.rs] |
| IT-21 | `key::""` (empty key) → exit 1 | N | 1 | F1=empty | [mutation_settings_set_test.rs] |
| IT-4 | `dry::2` → exit 1, out-of-range boolean | N | 1 | F3=2 | new |
| IT-5 | `bogus::x` → exit 1, unknown param | N | 1 | F6=present | new |
| IT-6 | `key::foo` without `value::` → exit 1, value required | N | 1 | F2=absent | new |
| IT-7 | Creates settings.json when file is absent | P | 0 | F4=missing | new |
| IT-8 | Updates existing key without duplication | P | 0 | F4=existing-with-key | new |
| IT-12 | `value::""` (shell-empty) → exit 1, rejected | N | 1 | F1=set, F2=empty-string | [mutation_settings_set_test.rs] |
| IT-22 | `dry::1` does not bypass empty-value rejection | N | 1 | F2=empty, F3=1 | [mutation_settings_set_test.rs] |

### Summary

- **Total:** 22 tests (11 positive, 11 negative)
- **Negative ratio:** 50.0% ✅ (≥40%)
- **IT range:** IT-1 to IT-22

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success (write or dry-run) | IT-1 through IT-3, IT-7 through IT-11, IT-13 through IT-15 |
| 1 | Invalid arguments | IT-4 through IT-6, IT-12, IT-16 through IT-19, IT-21 through IT-22 |
| 2 | Runtime error (HOME missing) | IT-20 |

### Type Inference Coverage (FR-07)

| Input string | JSON type | Test |
|-------------|-----------|------|
| `"true"` | boolean `true` | IT-1 |
| `"false"` | boolean `false` | IT-9 |
| `"0"` | integer `0` | IT-2 |
| `"42"` | integer `42` | IT-10 |
| `"hello"` | string `"hello"` | IT-11 |

**Type precedence** (FR-07): boolean check → i64 check → f64 check → string.
`"0"` is integer not boolean because i64 check precedes boolean check for numeric strings.
Empty `value::` is not a type-inference outcome — it is rejected before type inference runs (FR-04; see IT-12).

### Atomic Write Requirement (FR-06)

Writes use temp-file rename (`settings.json.tmp` → `settings.json`).
IT-14 verifies no duplication on update (not two copies of key).
IT-15 verifies append to existing file without corruption.

---

## Test Case Details

---

### IT-1: `value::true` → boolean `true`

- **Given:** `HOME=<tmp>`; settings absent.
- **When:**
  `clv .settings.set key::flag value::true`
- **Expected:** Exit 0; `settings.json` has `"flag": true` (unquoted).
- **Then:** boolean stored
- **Exit:** 0

---

### IT-2: `value::0` → integer `0`, not boolean

- **Given:** `HOME=<tmp>`.
- **When:**
  `clv .settings.set key::n value::0`
- **Expected:** Exit 0; `settings.json` has `"n": 0` (integer).
- **Then:** integer stored
- **Exit:** 0

---

### IT-3: `dry::1` → no file change

- **Given:** `HOME=<tmp>`; settings absent.
- **When:**
  `clv .settings.set key::k value::v dry::1`
- **Expected:** Exit 0; stdout contains `[dry-run]`; settings.json not created.
- **Then:** no side effects
- **Exit:** 0

---

### IT-4: `dry::2` → exit 1

- **Given:** clean environment
- **When:**
  `clv .settings.set key::k value::v dry::2`
- **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-5: `bogus::x` → exit 1

- **Given:** clean environment
- **When:**
  `clv .settings.set key::k value::v bogus::x`
- **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-6: `key::foo` without `value::` → exit 1

- **Given:** clean environment
- **When:** `clv .settings.set key::foo`
- **Then:** Exit 1; error message indicates `value::` is required
- **Exit:** 1
- **Source:** [command/readme.md](../../../../docs/cli/command/readme.md)

---

### IT-7: Creates settings.json when file is absent

- **Given:** `HOME=<tmp>` with no settings.json present
- **When:** `clv .settings.set key::theme value::dark`
- **Then:** Exit 0; settings.json created at `~/.claude/settings.json`; contains `"theme": "dark"`
- **Exit:** 0
- **Source:** [command/readme.md](../../../../docs/cli/command/readme.md)

---

### IT-8: Updates existing key without duplication

- **Given:** `HOME=<tmp>` with settings.json containing `"theme": "light"`
- **When:** `clv .settings.set key::theme value::dark`
- **Then:** Exit 0; settings.json now contains `"theme": "dark"` exactly once (no duplication)
- **Exit:** 0
- **Source:** [command/readme.md](../../../../docs/cli/command/readme.md)

---

### IT-9: `value::false` → boolean `false`

- **Given:** `HOME=<tmp>`; settings absent
- **When:** `clv .settings.set key::flag value::false`
- **Then:** exit 0; settings.json has `"flag": false` (unquoted boolean)
- **Exit:** 0
- **Source:** [mutation_settings_set_test.rs]

---

### IT-10: `value::42` → integer `42`

- **Given:** `HOME=<tmp>`; settings absent
- **When:** `clv .settings.set key::count value::42`
- **Then:** exit 0; settings.json has `"count": 42` (unquoted integer)
- **Exit:** 0
- **Source:** [mutation_settings_set_test.rs]

---

### IT-11: `value::hello` → string `"hello"`

- **Given:** `HOME=<tmp>`; settings absent
- **When:** `clv .settings.set key::name value::hello`
- **Then:** exit 0; settings.json has `"name": "hello"` (quoted string)
- **Exit:** 0
- **Source:** [mutation_settings_set_test.rs]

---

### IT-12: `value::""` (shell-empty) → exit 1, rejected

- **Given:** `HOME=<tmp>`; settings absent
- **When:** `clv .settings.set key::empty value::""` (shell-empty value, equivalent to `value::` with nothing after)
- **Then:** exit 1; settings.json NOT created; error mentions `value::`
- **Exit:** 1
- **Source:** [param/07_value.md — EC-10](../../../../docs/cli/param/07_value.md)
- **Note:** Bug-driven correction: this case previously claimed exit 0/stored-as-empty-string, contradicting `tc327_settings_set_empty_value_rejected` (the actual passing implementation). Gap Class — a spec case whose documented outcome is not backed by any test exercising that exact scenario, whether the case hedges between outcomes, confidently asserts an outcome contradicted by real behavior, confidently asserts an outcome that happens to be correct but unverified, or is missing from the spec's index entirely despite a passing implementation test existing for it. In every variant, the spec's authoritative record cannot be trusted to catch a future regression in that exact scenario. Source: BUG-006.

---

### IT-13: Creates file when settings.json absent

- **Given:** `HOME=<tmp>` with no `.claude/` directory or settings.json
- **When:** `clv .settings.set key::theme value::dark`
- **Then:** exit 0; settings.json created at `~/.claude/settings.json`; contains `"theme": "dark"`
- **Exit:** 0
- **Source:** [mutation_settings_set_test.rs]

---

### IT-14: Updates existing key without duplication

- **Given:** `HOME=<tmp>`; settings.json contains `"theme": "light"` and another key
- **When:** `clv .settings.set key::theme value::dark`
- **Then:** exit 0; settings.json has `"theme": "dark"` exactly once; other keys unchanged
- **Exit:** 0
- **Source:** [mutation_settings_set_test.rs]

---

### IT-15: Adds new key to existing file

- **Given:** `HOME=<tmp>`; settings.json exists with one key
- **When:** `clv .settings.set key::newKey value::newVal`
- **Then:** exit 0; settings.json has both old key and `"newKey": "newVal"`; no corruption
- **Exit:** 0
- **Source:** [mutation_settings_set_test.rs]

---

### IT-16: Without `key::` → error mentions `key::`

- **Given:** `HOME=<tmp>` with valid settings.json
- **When:** `clv .settings.set value::dark`
- **Then:** exit 1; error message contains the string `key::`
- **Exit:** 1
- **Source:** [read_settings_test.rs]

---

### IT-17: `key::foo` without `value::` → error mentions `value::`

- **Given:** `HOME=<tmp>` with valid settings.json
- **When:** `clv .settings.set key::foo`
- **Then:** exit 1; error message contains the string `value::`
- **Exit:** 1
- **Source:** [read_settings_test.rs]

---

### IT-18: No `key::` → exit 1

- **Given:** clean environment
- **When:** `clv .settings.set`
- **Then:** exit 1; error: `key::` is required
- **Exit:** 1
- **Source:** [mutation_settings_set_test.rs]

---

### IT-19: `key::` present but no `value::` → exit 1

- **Given:** clean environment
- **When:** `clv .settings.set key::theme`
- **Then:** exit 1; error: `value::` is required
- **Exit:** 1
- **Source:** [mutation_settings_set_test.rs]

---

### IT-20: HOME not set → exit 2

- **Given:** HOME environment variable is unset
- **When:** `clv .settings.set key::theme value::dark`
- **Then:** exit 2; error references HOME or settings path resolution failure
- **Exit:** 2
- **Source:** [mutation_settings_set_test.rs]

---

### IT-21: `key::""` (empty key) → exit 1

- **Given:** clean environment
- **When:** `clv .settings.set key:: value::dark`
- **Then:** exit 1; error: key value cannot be empty
- **Exit:** 1
- **Source:** [mutation_settings_set_test.rs]

---

### IT-22: `dry::1` does not bypass empty-value rejection

- **Given:** clean environment
- **When:** `clv .settings.set key::k value:: dry::1`
- **Then:** exit 1; error mentions `value::`; no file created; validation fires before the dry-run preview branch
- **Exit:** 1
- **Source:** [param/07_value.md — EC-10](../../../../docs/cli/param/07_value.md)
- **Note:** Bug-driven expansion: implemented regression test `tc334_settings_set_empty_value_with_dry_still_rejected` had no corresponding documented case in this spec's Test Case Index. Gap Class — a spec case whose documented outcome is not backed by any test exercising that exact scenario, whether the case hedges between outcomes, confidently asserts an outcome contradicted by real behavior, confidently asserts an outcome that happens to be correct but unverified, or is missing from the spec's index entirely despite a passing implementation test existing for it. In every variant, the spec's authoritative record cannot be trusted to catch a future regression in that exact scenario. Source: BUG-006.

---

### Source Functions

| Function | File |
|----------|------|
| `tc320_settings_set_missing_key_exits_1` | `tests/cli/mutation_settings_set_test.rs` |
| `tc321_settings_set_missing_value_exits_1` | `tests/cli/mutation_settings_set_test.rs` |
| `tc322_settings_set_stores_boolean_true` | `tests/cli/mutation_settings_set_test.rs` |
| `tc323_settings_set_stores_boolean_false` | `tests/cli/mutation_settings_set_test.rs` |
| `tc324_settings_set_zero_stored_as_number` | `tests/cli/mutation_settings_set_test.rs` |
| `tc325_settings_set_stores_number` | `tests/cli/mutation_settings_set_test.rs` |
| `tc326_settings_set_stores_string` | `tests/cli/mutation_settings_set_test.rs` |
| `tc327_settings_set_empty_value_rejected` | `tests/cli/mutation_settings_set_test.rs` |
| `tc328_settings_set_creates_file_when_absent` | `tests/cli/mutation_settings_set_test.rs` |
| `tc329_settings_set_updates_existing_key` | `tests/cli/mutation_settings_set_test.rs` |
| `tc330_settings_set_dry_shows_preview_no_write` | `tests/cli/mutation_settings_set_test.rs` |
| `tc331_settings_set_no_home_exits_2` | `tests/cli/mutation_settings_set_test.rs` |
| `tc332_settings_set_empty_key_exits_1` | `tests/cli/mutation_settings_set_test.rs` |
| `tc333_settings_set_adds_new_key_preserves_existing` | `tests/cli/mutation_settings_set_test.rs` |
| `tc334_settings_set_empty_value_with_dry_still_rejected` | `tests/cli/mutation_settings_set_test.rs` |
| `tc238_settings_set_missing_key_error_format` | `tests/cli/read_settings_test.rs` |
| `tc239_settings_set_missing_value_error_format` | `tests/cli/read_settings_test.rs` |
| `tc252_settings_set_dry_no_write` | `tests/cli/cross_cutting_test.rs` |
| `it04_settings_set_dry2_exits_1` | `tests/cli/mutation_settings_set_test.rs` |
| `it05_settings_set_bogus_param_exits_1` | `tests/cli/mutation_settings_set_test.rs` |
| `it06_settings_set_key_without_value_exits_1` | `tests/cli/mutation_settings_set_test.rs` |
| `tc506_settings_set_missing_value_error_contains_value` | `tests/cli/error_messages_test.rs` |

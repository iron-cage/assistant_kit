# Test: `.settings.set`

Integration test planning for the `.settings.set` command. See [commands.md](../../commands.md) for specification.

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
| TC-322 | `value::true` → stores boolean `true` | P | 0 | F1=set, F2=true | [mutation_commands_test.rs] |
| TC-323 | `value::false` → stores boolean `false` | P | 0 | F1=set, F2=false | [mutation_commands_test.rs] |
| TC-324 | `value::0` → stores number `0` (NOT boolean) | P | 0 | F1=set, F2=0 | [mutation_commands_test.rs] |
| TC-325 | `value::42` → stores integer `42` | P | 0 | F1=set, F2=int | [mutation_commands_test.rs] |
| TC-326 | `value::hello` → stores quoted `"hello"` | P | 0 | F1=set, F2=string | [mutation_commands_test.rs] |
| TC-327 | `value::""` → stores empty string `""` | P | 0 | F1=set, F2=empty-string | [mutation_commands_test.rs] |
| TC-328 | Creates file when settings.json absent | P | 0 | F4=missing | [mutation_commands_test.rs] |
| TC-329 | Updates existing key without duplication | P | 0 | F4=existing-with-key | [mutation_commands_test.rs] |
| TC-330 | `dry::1` → shows preview, no file change | P | 0 | F3=1 | [mutation_commands_test.rs] |
| TC-333 | Adds new key to existing file | P | 0 | F4=existing-without-key | [mutation_commands_test.rs] |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| TC-238 | Without `key::` → error mentions `key::` | N | 1 | F1=absent | [read_commands_test.rs] |
| TC-239 | `key::foo` without `value::` → error mentions `value::` | N | 1 | F2=absent | [read_commands_test.rs] |
| TC-320 | No `key::` → exit 1 | N | 1 | F1=absent | [mutation_commands_test.rs] |
| TC-321 | `key::` present but no `value::` → exit 1 | N | 1 | F1=set, F2=absent | [mutation_commands_test.rs] |
| TC-331 | HOME not set → exit 2 | N | 2 | F5=empty | [mutation_commands_test.rs] |
| TC-332 | `key::""` (empty key) → exit 1 | N | 1 | F1=empty | [mutation_commands_test.rs] |
| TC-475 | `dry::2` → exit 1, out-of-range boolean | N | 1 | F3=2 | new |
| TC-476 | `bogus::x` → exit 1, unknown param | N | 1 | F6=present | new |

### Summary

- **Total:** 18 tests (10 positive, 8 negative)
- **Negative ratio:** 44.4% ✅ (≥40%)
- **TC range:** TC-238 to TC-476

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success (write or dry-run) | TC-322 through TC-330, TC-333 |
| 1 | Invalid arguments | TC-238, TC-239, TC-320, TC-321, TC-332, TC-475, TC-476 |
| 2 | Runtime error (HOME missing) | TC-331 |

### Type Inference Coverage (FR-07)

| Input string | JSON type | Test |
|-------------|-----------|------|
| `"true"` | boolean `true` | TC-322 |
| `"false"` | boolean `false` | TC-323 |
| `"0"` | integer `0` | TC-324 |
| `"42"` | integer `42` | TC-325 |
| `"hello"` | string `"hello"` | TC-326 |
| `""` | string `""` | TC-327 |

**Type precedence** (FR-07): boolean check → i64 check → f64 check → string.
`"0"` is integer not boolean because i64 check precedes boolean check for numeric strings.

### Atomic Write Requirement (FR-06)

Writes use temp-file rename (`settings.json.tmp` → `settings.json`).
TC-329 verifies no duplication on update (not two copies of key).
TC-333 verifies append to existing file without corruption.

---

## Test Case Details

### TC-322: `value::true` → boolean `true`

**Goal:** String "true" is type-inferred as JSON boolean.
**Setup:** `HOME=<tmp>`; settings absent.
**Command:** `cm .settings.set key::flag value::true`
**Expected:** Exit 0; `settings.json` has `"flag": true` (unquoted).
**Verification:** File content has unquoted `true`.
**Pass Criteria:** Exit 0; boolean stored.

---

### TC-324: `value::0` → integer `0`, not boolean

**Goal:** "0" stores as JSON number 0, not as boolean false (FR-07 precedence).
**Setup:** `HOME=<tmp>`.
**Command:** `cm .settings.set key::n value::0`
**Expected:** Exit 0; `settings.json` has `"n": 0` (integer).
**Verification:** File content has unquoted `0` (not `false`).
**Pass Criteria:** Exit 0; integer stored.

---

### TC-330: `dry::1` → no file change

**Goal:** Dry-run mode shows preview without writing.
**Setup:** `HOME=<tmp>`; settings absent.
**Command:** `cm .settings.set key::k value::v dry::1`
**Expected:** Exit 0; stdout contains `[dry-run]`; settings.json not created.
**Verification:** exit code 0; `[dry-run]` in output; file absent.
**Pass Criteria:** Exit 0; no side effects.

---

### TC-475: `dry::2` → exit 1

**Goal:** Boolean parameters only accept 0 or 1.
**Setup:** None.
**Command:** `cm .settings.set key::k value::v dry::2`
**Expected:** Exit 1.
**Verification:** exit code 1.
**Pass Criteria:** Exit 1.

---

### TC-476: `bogus::x` → exit 1

**Goal:** Unknown parameter rejected.
**Setup:** None.
**Command:** `cm .settings.set key::k value::v bogus::x`
**Expected:** Exit 1.
**Verification:** exit code 1.
**Pass Criteria:** Exit 1.

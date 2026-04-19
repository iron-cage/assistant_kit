# Test: `.settings.show`

Integration test planning for the `.settings.show` command. See [commands.md](../../../../../docs/cli/commands.md) for specification.

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
| TC-162 | Empty `{}` → empty output, exit 0 | P | 0 | F3=empty | [read_commands_test.rs] |
| TC-163 | Valid settings → keys shown, exit 0 | P | 0 | F3=valid | [read_commands_test.rs] |
| TC-164 | `v::0` → `key=value` format | P | 0 | F1=0, F3=valid | [read_commands_test.rs] |
| TC-167 | `format::json` → valid JSON object | P | 0 | F2=json, F3=valid | [read_commands_test.rs] |
| TC-241 | `format::json` preserves bool/number types | P | 0 | F2=json, F5=boolean | [read_commands_test.rs] |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| TC-161 | File missing → exit 2 | N | 2 | F3=missing | [read_commands_test.rs] |
| TC-170 | Malformed JSON → exit 2 | N | 2 | F3=malformed | [read_commands_test.rs] |
| TC-171 | HOME not set → exit 2 | N | 2 | F4=empty | [read_commands_test.rs] |
| TC-468 | `bogus::x` → exit 1 | N | 1 | F6=present | new |
| TC-469 | `format::xml` → exit 1 | N | 1 | F2=xml | new |
| TC-470 | `v::3` → exit 1, out of range | N | 1 | F1=3 | new |

### Summary

- **Total:** 11 tests (5 positive, 6 negative)
- **Negative ratio:** 54.5% ✅ (≥40%)
- **TC range:** TC-161 to TC-470

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success | TC-162, TC-163, TC-164, TC-167, TC-241 |
| 1 | Invalid arguments | TC-468, TC-469, TC-470 |
| 2 | Runtime error (missing file, bad JSON, no HOME) | TC-161, TC-170, TC-171 |

### Settings File State Coverage

| State | Tests |
|-------|-------|
| Missing | TC-161 (exit 2) |
| Empty `{}` | TC-162 (exit 0, empty output) |
| Valid with keys | TC-163, TC-164, TC-167 |
| Malformed | TC-170 (exit 2) |

---

## Test Case Details

### TC-161: File missing → exit 2

**Goal:** Missing settings file is a runtime error.
**Setup:** `HOME=<tmp>` with no `.claude/settings.json`.
**Command:** `cm .settings.show`
**Expected:** Exit 2.
**Verification:** exit code 2.
**Pass Criteria:** Exit 2.

---

### TC-162: Empty `{}` → empty output

**Goal:** Valid but empty settings file produces empty output (not an error).
**Setup:** `HOME=<tmp>`; `settings.json` = `{}`.
**Command:** `cm .settings.show`
**Expected:** Exit 0; stdout is empty.
**Verification:** exit code 0; stdout trimmed is empty.
**Pass Criteria:** Exit 0; no output.

---

### TC-163: Valid settings → keys shown

**Goal:** Keys and values appear in output.
**Setup:** `HOME=<tmp>`; `settings.json` has `myKey = "myValue"`.
**Command:** `cm .settings.show`
**Expected:** Exit 0; output contains "myKey" and "myValue".
**Verification:** output contains key name and value.
**Pass Criteria:** Exit 0; key/value visible.

---

### TC-241: `format::json` preserves bool/number types

**Goal:** JSON output uses native types, not quoted strings.
**Setup:** `HOME=<tmp>`; settings has boolean and integer values.
**Command:** `cm .settings.show format::json`
**Expected:** Exit 0; JSON output has `true`/`false` booleans and numeric integers (not quoted strings).
**Verification:** Output contains unquoted `true`, `false`, or integer literals.
**Pass Criteria:** Exit 0; type preservation.

---

### TC-468: `bogus::x` → exit 1

**Goal:** Unknown parameter rejected.
**Setup:** None.
**Command:** `cm .settings.show bogus::x`
**Expected:** Exit 1.
**Verification:** exit code 1.
**Pass Criteria:** Exit 1.

---

### TC-469: `format::xml` → exit 1

**Goal:** Unrecognized format rejected.
**Setup:** None.
**Command:** `cm .settings.show format::xml`
**Expected:** Exit 1.
**Verification:** exit code 1.
**Pass Criteria:** Exit 1.

---

### TC-470: `v::3` → exit 1

**Goal:** Out-of-range verbosity rejected.
**Setup:** None.
**Command:** `cm .settings.show v::3`
**Expected:** Exit 1.
**Verification:** exit code 1.
**Pass Criteria:** Exit 1.

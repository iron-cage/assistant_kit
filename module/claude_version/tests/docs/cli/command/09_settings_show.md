# Test: `.settings.show`

Integration test planning for the `.settings.show` command. See [commands.md](../../../../docs/cli/commands.md) for specification.

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
| TC-164 | `v::0` → `key=value` format | P | 0 | F1=0, F3=valid | [read_commands_test.rs] |
| TC-167 | `format::json` → valid JSON object | P | 0 | F2=json, F3=valid | [read_commands_test.rs] |
| IT-4 | `format::json` preserves bool/number types | P | 0 | F2=json, F5=boolean | [read_commands_test.rs] |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-1 | File missing → exit 2 | N | 2 | F3=missing | [read_commands_test.rs] |
| TC-170 | Malformed JSON → exit 2 | N | 2 | F3=malformed | [read_commands_test.rs] |
| TC-171 | HOME not set → exit 2 | N | 2 | F4=empty | [read_commands_test.rs] |
| IT-5 | `bogus::x` → exit 1 | N | 1 | F6=present | new |
| IT-6 | `format::xml` → exit 1 | N | 1 | F2=xml | new |
| IT-7 | `v::3` → exit 1, out of range | N | 1 | F1=3 | new |
| IT-8 | Output goes to stdout only; stderr is empty | P | 0 | F3=valid, F4=set | new |

### Summary

- **Total:** 12 tests (6 positive, 6 negative)
- **Negative ratio:** 50.0% ✅ (≥40%)
- **TC range:** IT-1 to IT-8

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success | IT-2, IT-3, TC-164, TC-167, IT-4 |
| 1 | Invalid arguments | IT-5, IT-6, IT-7 |
| 2 | Runtime error (missing file, bad JSON, no HOME) | IT-1, TC-170, TC-171 |

### Settings File State Coverage

| State | Tests |
|-------|-------|
| Missing | IT-1 (exit 2) |
| Empty `{}` | IT-2 (exit 0, empty output) |
| Valid with keys | IT-3, TC-164, TC-167 |
| Malformed | TC-170 (exit 2) |

---

## Test Case Details

---

### IT-1: File missing → exit 2

- **Given:** `HOME=<tmp>` with no `.claude/settings.json`.
- **When:**
  `cm .settings.show`
  **Expected:** Exit 2.
- **Then:** see spec
- **Exit:** 2

---

### IT-2: Empty `{}` → empty output

- **Given:** `HOME=<tmp>`; `settings.json` = `{}`.
- **When:**
  `cm .settings.show`
  **Expected:** Exit 0; stdout is empty.
- **Then:** no output
- **Exit:** 0

---

### IT-3: Valid settings → keys shown

- **Given:** `HOME=<tmp>`; `settings.json` has `myKey = "myValue"`.
- **When:**
  `cm .settings.show`
  **Expected:** Exit 0; output contains "myKey" and "myValue".
- **Then:** key/value visible
- **Exit:** 0

---

### IT-4: `format::json` preserves bool/number types

- **Given:** `HOME=<tmp>`; settings has boolean and integer values.
- **When:**
  `cm .settings.show format::json`
  **Expected:** Exit 0; JSON output has `true`/`false` booleans and numeric integers (not quoted strings).
- **Then:** type preservation
- **Exit:** 0

---

### IT-5: `bogus::x` → exit 1

- **Given:** clean environment
- **When:**
  `cm .settings.show bogus::x`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-6: `format::xml` → exit 1

- **Given:** clean environment
- **When:**
  `cm .settings.show format::xml`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-7: `v::3` → exit 1

- **Given:** clean environment
- **When:**
  `cm .settings.show v::3`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-8: Output goes to stdout only; stderr is empty

- **Given:** `HOME=<tmp>` with valid settings.json containing at least one key
- **When:** `cm .settings.show`
- **Then:** stdout is non-empty; stderr is empty
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

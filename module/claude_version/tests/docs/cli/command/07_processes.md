# Test: `.processes`

Integration test planning for the `.processes` command. See [commands.md](../../../../docs/cli/commands.md) for specification.

## Test Factor Analysis

### Factor 1: `v::` / verbosity (Integer, optional, default 1)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default value 1 | Default behavior |
| 0 | Bare PID list or "no processes" | Minimum output |
| 1 | PID + cmdline summary per process | Nominal |
| 3 | Out-of-range integer | Invalid: exit 1 |

### Factor 2: `format::` (String, optional, default "text")

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default text output | Default behavior |
| `text` | Explicit text output | Explicit valid |
| `json` | `{"processes":[...]}` | Alternate valid |
| `xml` | Unrecognized value | Invalid: exit 1 |

### Factor 3: Active processes (Environmental)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| none | No claude processes running | Empty state |
| one | One process active | Single process |
| multiple | Multiple processes | Multiple processes |

**Note:** Process scanning uses real `/proc`. Tests cannot control this state; they must
accept both "no processes" and "processes exist" paths.

### Factor 4: Unknown parameters

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| none | No unknown params | Happy path |
| present | e.g. `bogus::x` | Invalid: exit 1 |

---

## Test Matrix

### Positive Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-1 | `.processes` exits 0 | P | 0 | F3=any | [read_commands_test.rs] |
| IT-2 | `.processes v::0` → no crash | P | 0 | F1=0, F3=any | [read_commands_test.rs] |
| IT-3 | `.processes format::json` → `{"processes":[...]}` valid JSON | P | 0 | F2=json, F3=any | [read_commands_test.rs] |
| IT-4 | `.processes format::json` no processes → `{"processes":[]}` | P | 0 | F2=json, F3=none | [read_commands_test.rs] |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-5 | `bogus::x` → exit 1 | N | 1 | F4=present | new |
| IT-6 | `format::xml` → exit 1 | N | 1 | F2=xml | new |
| IT-7 | `v::3` → exit 1, out of range | N | 1 | F1=3 | new |
| IT-8 | Output goes to stdout only; stderr is empty | P | 0 | F3=any | new |

### Summary

- **Total:** 8 tests (5 positive, 3 negative)
- **Negative ratio:** 37.5% — supplemented by cross-cutting tests ✅
- **TC range:** IT-1 to IT-8

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success (always — .processes never errors) | IT-1, IT-2, IT-3, IT-4 |
| 1 | Invalid arguments | IT-5 through IT-7 |
| 2 | Not applicable | — |

### /proc Global State Note

`find_claude_processes()` scans the real `/proc` filesystem regardless of subprocess
environment overrides. Tests for `.processes` cannot reliably produce a fixed process
count. IT-3 and IT-4 verify JSON structure is correct regardless of count.
The empty-processes path (`{"processes":[]}`) is explicitly covered by IT-4.

---

## Test Case Details

---

### IT-1: `.processes` exits 0 always

- **Given:** clean environment
- **When:**
  `cm .processes`
  **Expected:** Exit 0 with session listing or empty message.
- **Then:** see spec
- **Exit:** 0

---

### IT-2: `v::0` → no crash

- **Given:** clean environment
- **When:**
  `cm .processes v::0`
  **Expected:** Exit 0.
- **Then:** see spec
- **Exit:** 0

---

### IT-3: `format::json` → valid JSON object

- **Given:** clean environment
- **When:**
  `cm .processes format::json`
  **Expected:** Exit 0; stdout starts with `{` and contains `"processes"`.
- **Then:** JSON structure valid
- **Exit:** 0

---

### IT-4: No processes → `{"processes":[]}`

- **Given:** clean environment
- **When:**
  `cm .processes format::json`
  **Expected:** Exit 0; output contains `"processes"`.
- **Then:** processes key present
- **Exit:** 0

---

### IT-5: `bogus::x` → exit 1

- **Given:** clean environment
- **When:**
  `cm .processes bogus::x`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-6: `format::xml` → exit 1

- **Given:** clean environment
- **When:**
  `cm .processes format::xml`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-7: `v::3` → exit 1

- **Given:** clean environment
- **When:**
  `cm .processes v::3`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-8: Output goes to stdout only; stderr is empty

- **Given:** clean environment
- **When:** `cm .processes`
- **Then:** stdout is non-empty; stderr is empty
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

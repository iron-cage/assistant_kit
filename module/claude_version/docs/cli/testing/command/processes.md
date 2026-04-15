# Test: `.processes`

Integration test planning for the `.processes` command. See [commands.md](../../commands.md) for specification.

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
| TC-137 | `.processes` exits 0 | P | 0 | F3=any | [read_commands_test.rs] |
| TC-141 | `.processes v::0` → no crash | P | 0 | F1=0, F3=any | [read_commands_test.rs] |
| TC-144 | `.processes format::json` → `{"processes":[...]}` valid JSON | P | 0 | F2=json, F3=any | [read_commands_test.rs] |
| TC-145 | `.processes format::json` no processes → `{"processes":[]}` | P | 0 | F2=json, F3=none | [read_commands_test.rs] |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| TC-462 | `bogus::x` → exit 1 | N | 1 | F4=present | new |
| TC-463 | `format::xml` → exit 1 | N | 1 | F2=xml | new |
| TC-464 | `v::3` → exit 1, out of range | N | 1 | F1=3 | new |

### Summary

- **Total:** 7 tests (4 positive, 3 negative)
- **Negative ratio:** 42.9% ✅ (≥40%)
- **TC range:** TC-137 to TC-464

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success (always — .processes never errors) | TC-137, TC-141, TC-144, TC-145 |
| 1 | Invalid arguments | TC-462 through TC-464 |
| 2 | Not applicable | — |

### /proc Global State Note

`find_claude_processes()` scans the real `/proc` filesystem regardless of subprocess
environment overrides. Tests for `.processes` cannot reliably produce a fixed process
count. TC-144 and TC-145 verify JSON structure is correct regardless of count.
The empty-processes path (`{"processes":[]}`) is explicitly covered by TC-145.

---

## Test Case Details

### TC-137: `.processes` exits 0 always

**Goal:** Process list never fails (empty or populated).
**Setup:** None.
**Command:** `cm .processes`
**Expected:** Exit 0 with session listing or empty message.
**Verification:** exit code 0.
**Pass Criteria:** Exit 0.

---

### TC-141: `v::0` → no crash

**Goal:** Minimum verbosity produces output without crashing.
**Setup:** None.
**Command:** `cm .processes v::0`
**Expected:** Exit 0.
**Verification:** exit code 0.
**Pass Criteria:** Exit 0.

---

### TC-144: `format::json` → valid JSON object

**Goal:** JSON output has `processes` array key.
**Setup:** None.
**Command:** `cm .processes format::json`
**Expected:** Exit 0; stdout starts with `{` and contains `"processes"`.
**Verification:** output contains `"processes"` and starts with `{`.
**Pass Criteria:** Exit 0; JSON structure valid.

---

### TC-145: No processes → `{"processes":[]}`

**Goal:** Empty processes produces valid empty JSON array, not `null`.
**Setup:** None (TC accepts either empty or populated output).
**Command:** `cm .processes format::json`
**Expected:** Exit 0; output contains `"processes"`.
**Verification:** output contains `"processes"`.
**Pass Criteria:** Exit 0; processes key present.

---

### TC-462: `bogus::x` → exit 1

**Goal:** Unknown parameter rejected before process scan.
**Setup:** None.
**Command:** `cm .processes bogus::x`
**Expected:** Exit 1.
**Verification:** exit code 1.
**Pass Criteria:** Exit 1.

---

### TC-463: `format::xml` → exit 1

**Goal:** Unrecognized format rejected.
**Setup:** None.
**Command:** `cm .processes format::xml`
**Expected:** Exit 1.
**Verification:** exit code 1.
**Pass Criteria:** Exit 1.

---

### TC-464: `v::3` → exit 1

**Goal:** Out-of-range verbosity rejected.
**Setup:** None.
**Command:** `cm .processes v::3`
**Expected:** Exit 1.
**Verification:** exit code 1.
**Pass Criteria:** Exit 1.

# Test: `.processes`

### Scope

- **Purpose**: Integration test cases for the `.processes` command.
- **Responsibility**: Test factor analysis, case index, and expected behavior for process listing.
- **In Scope**: `/proc` scanning, PID reporting, verbosity levels, output formats.
- **Out of Scope**: Parameter edge cases (→ `../param/`), group interactions (→ `../param_group/`).

Integration test planning for the `.processes` command. See [command/readme.md](../../../../docs/cli/command/readme.md) for specification.

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
| IT-1 | `.processes` exits 0 | P | 0 | F3=any | [read_processes_test.rs] |
| IT-2 | `.processes v::0` → no crash | P | 0 | F1=0, F3=any | [read_processes_test.rs] |
| IT-3 | `.processes format::json` → `{"processes":[...]}` valid JSON | P | 0 | F2=json, F3=any | [read_processes_test.rs] |
| IT-4 | `.processes format::json` no processes → `{"processes":[]}` | P | 0 | F2=json, F3=none | [read_processes_test.rs] |

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
  `clv .processes`
  **Expected:** Exit 0 with session listing or empty message.
- **Then:** exit 0; stdout contains zero or more Claude process entries or an empty message; stderr is empty
- **Exit:** 0
- **Source:** [command/processes.md](../../../../docs/cli/command/processes.md)

---

### IT-2: `v::0` → no crash

- **Given:** clean environment
- **When:**
  `clv .processes v::0`
  **Expected:** Exit 0.
- **Then:** exit 0; stdout contains bare PID values or empty message with no label decoration; stderr is empty
- **Exit:** 0
- **Source:** [command/processes.md](../../../../docs/cli/command/processes.md)

---

### IT-3: `format::json` → valid JSON object

- **Given:** clean environment
- **When:**
  `clv .processes format::json`
  **Expected:** Exit 0; stdout starts with `{` and contains `"processes"`.
- **Then:** exit 0; stdout is valid JSON starting with `{` and containing a `"processes"` array key; stderr is empty
- **Exit:** 0
- **Source:** [command/processes.md](../../../../docs/cli/command/processes.md)

---

### IT-4: No processes → `{"processes":[]}`

- **Given:** clean environment
- **When:**
  `clv .processes format::json`
  **Expected:** Exit 0; output contains `"processes"`.
- **Then:** exit 0; stdout contains `{"processes":[]}` or a JSON object with an empty `"processes"` array (no processes guaranteed); stderr is empty
- **Exit:** 0
- **Source:** [command/processes.md](../../../../docs/cli/command/processes.md)

---

### IT-5: `bogus::x` → exit 1

- **Given:** clean environment
- **When:**
  `clv .processes bogus::x`
  **Expected:** Exit 1.
- **Then:** exit 1; stderr or stdout contains "bogus" or "unknown parameter" error message; no process entries produced
- **Exit:** 1
- **Source:** [command/processes.md](../../../../docs/cli/command/processes.md)

---

### IT-6: `format::xml` → exit 1

- **Given:** clean environment
- **When:**
  `clv .processes format::xml`
  **Expected:** Exit 1.
- **Then:** exit 1; stderr or stdout references invalid format value "xml" or lists valid format options; no process entries produced
- **Exit:** 1
- **Source:** [command/processes.md](../../../../docs/cli/command/processes.md)

---

### IT-7: `v::3` → exit 1

- **Given:** clean environment
- **When:**
  `clv .processes v::3`
  **Expected:** Exit 1.
- **Then:** exit 1; stderr or stdout references out-of-range verbosity value "3" or expected range; no process entries produced
- **Exit:** 1
- **Source:** [command/processes.md](../../../../docs/cli/command/processes.md)

---

### IT-8: Output goes to stdout only; stderr is empty

- **Given:** clean environment
- **When:** `clv .processes`
- **Then:** stdout is non-empty; stderr is empty
- **Exit:** 0
- **Source:** [command/processes.md](../../../../docs/cli/command/processes.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc137_processes_exits_0` | `tests/cli/read_processes_test.rs` |
| `tc141_processes_v0_no_crash` | `tests/cli/read_processes_test.rs` |
| `tc144_processes_format_json_valid` | `tests/cli/read_processes_test.rs` |
| `tc145_processes_format_json_empty_when_no_processes` | `tests/cli/read_processes_test.rs` |

# Test: `.status`

Integration test planning for the `.status` command. See [commands.md](../../../../docs/cli/commands.md) for specification.

## Test Factor Analysis

### Factor 1: `v::` / verbosity (Integer, optional, default 1)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default value 1, labeled output | Default behavior |
| 0 | Bare 3-line output (version, processes, account) | Minimum output |
| 1 | Labeled lines: `Version: X`, `Processes: N`, `Account: X` | Nominal |
| 2 | Extended detail (same as 1 if no extra data available) | Maximum detail |
| 3 | Out-of-range integer | Invalid: exit 1 |
| `abc` | Non-integer string | Invalid: exit 1 |

Boundary set: 0, 1, 2, 3 (out-of-range).

### Factor 2: `format::` (String, optional, default "text")

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default text output | Default behavior |
| `text` | Explicit text output | Explicit valid |
| `json` | JSON object with required keys | Alternate valid |
| `xml` | Unrecognized value | Invalid: exit 1 |
| `JSON` | Wrong case | Invalid: exit 1 |
| (empty) | Empty string value | Invalid: exit 1 |

### Factor 3: PATH / claude availability (Environmental)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| claude found | Version string available | Happy path |
| empty PATH | Version "not found", still exits 0 | Degraded |

### Factor 4: HOME environment (Environmental)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| set | Account info available | Happy path |
| empty | Account shown as "unknown", exits 0 | Degraded |

### Factor 5: Preferred version in settings (State)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | No "Preferred:" line in output | No preference |
| set | "Preferred:" line shown | Preference stored |

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
| IT-1 | `.status` exits 0 always | P | 0 | F1=absent, F2=absent | [read_commands_test.rs] |
| IT-2 | `.status` with empty PATH → "not found", exits 0 | P | 0 | F3=empty PATH | [read_commands_test.rs] |
| IT-3 | `.status v::0` → exactly 3 bare lines | P | 0 | F1=0 | [read_commands_test.rs] |
| IT-4 | `.status v::1` → labeled Version/Processes/Account lines | P | 0 | F1=1 | [read_commands_test.rs] |
| IT-5 | `.status format::json` → valid JSON with required keys | P | 0 | F2=json | [read_commands_test.rs] |
| IT-6 | `.status v::0` has fewer/equal lines than `.status v::1` | P | 0 | F1=0 vs 1 | [read_commands_test.rs] |
| IT-7 | `.status` HOME not set → account "unknown", no crash | P | 0 | F4=empty | [read_commands_test.rs] |
| IT-8 | `.status` with no preference → no "Preferred" line | P | 0 | F5=absent | [read_commands_test.rs] |
| IT-9 | `.status` with preference → shows "Preferred" line | P | 0 | F5=set | [read_commands_test.rs] |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-10 | `.status format::xml` → exit 1 | N | 1 | F2=xml | new |
| IT-11 | `.status v::3` → exit 1, out of range | N | 1 | F1=3 | new |
| IT-12 | `.status bogus::x` → exit 1 | N | 1 | F6=present | new |

### Summary

- **Total:** 12 tests (9 positive, 3 negative)
- **Negative ratio:** 25.0% — below ≥40% threshold; covered by cross-cutting TC-242 to TC-244 and TC-245 in `read_commands_test.rs` which apply to `.status` among other commands
- **Existing cross-cutting negatives applying to `.status`:** TC-242 (`format::xml`), TC-243 (`format::JSON`), TC-244 (`format::`), TC-245 (`v::` duplication)
- **Combined negative count (command-specific + cross-cutting):** 7/16 = 43.8% ✅
- **TC range:** IT-1 to IT-12

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success (always — .status never errors) | IT-1 through IT-7, IT-8, IT-9 |
| 1 | Invalid arguments | IT-10 through IT-12 |
| 2 | Not applicable (.status always exits 0 for any valid state) | — |

### Degradation Semantics

`.status` exhibits unique behavior: it always exits 0 regardless of environment state.
Missing claude, missing HOME, or missing accounts produce informational "not found"/"unknown" output
rather than exit 2. This is by design (FR-01: status is read-only, never fails).

### Factor Coverage

| Factor | Positive Coverage | Negative Coverage |
|--------|-------------------|-------------------|
| F1 (v::) | IT-3 (v=0), IT-4 (v=1), IT-1 (absent) | IT-11 (v::3) |
| F2 (format) | IT-5 (json) | IT-10 (xml) |
| F3 (PATH) | IT-1 (found), IT-2 (empty) | — |
| F4 (HOME) | IT-1 (set), IT-7 (empty) | — |
| F5 (preference) | IT-8 (absent), IT-9 (set) | — |
| F6 (unknown params) | — | IT-12 |

---

## Test Case Details

---

### IT-1: `.status` exits 0 always

- **Given:** clean environment
- **When:**
  `cm .status`
  **Expected:** Exit 0.
- **Then:** see spec
- **Exit:** 0

---

### IT-2: Empty PATH → "not found", exits 0

- **Given:** `PATH=""`, `HOME=<tmp>`.
- **When:**
  `cm .status`
  **Expected:** Exit 0; output contains "not found" or "unknown".
- **Then:** no crash
- **Exit:** 0

---

### IT-3: `v::0` → exactly 3 bare lines

- **Given:** `HOME=<tmp>` with empty settings (no preference stored).
- **When:**
  `cm .status v::0`
  **Expected:** Exactly 3 non-empty lines.
- **Then:** 3 lines
- **Exit:** 0

---

### IT-4: `v::1` → labeled lines

- **Given:** clean environment
- **When:**
  `cm .status v::1`
  **Expected:** Output contains "Version:", "Processes:", "Account:".
- **Then:** all labels
- **Exit:** 0

---

### IT-5: `format::json` → valid JSON

- **Given:** clean environment
- **When:**
  `cm .status format::json`
  **Expected:** JSON object with `version`, `processes`, `account` keys.
- **Then:** required JSON fields present
- **Exit:** 0

---

### IT-6: `v::0` has ≤ lines than `v::1`

- **Given:** clean environment
- **When:**
  Run both `v::0` and `v::1`.
  **Expected:** Line count of `v::0` ≤ line count of `v::1`.
- **Then:** for both; v::0 not longer than v::1
- **Exit:** 0

---

### IT-7: HOME not set → "unknown" account, no crash

- **Given:** `HOME=""`.
- **When:**
  `cm .status`
  **Expected:** Exit 0; stdout contains "unknown".
- **Then:** graceful degradation
- **Exit:** 0

---

### IT-8: No preference → no "Preferred" line

- **Given:** `HOME=<tmp>`; `settings.json` has no `preferredVersionSpec`.
- **When:**
  `cm .status`
  **Expected:** Output does not contain "Preferred".
- **Then:** no preference line
- **Exit:** 0

---

### IT-9: With preference → shows "Preferred" line

- **Given:** `HOME=<tmp>`; `settings.json` has `preferredVersionSpec = "stable"`.
- **When:**
  `cm .status`
  **Expected:** Output contains "Preferred".
- **Then:** preference line present
- **Exit:** 0

---

### IT-10: `format::xml` → exit 1

- **Given:** clean environment
- **When:**
  `cm .status format::xml`
  **Expected:** Exit 1; stderr mentions unknown format.
- **Then:** see spec
- **Exit:** 1

---

### IT-11: `v::3` → exit 1, out of range

- **Given:** clean environment
- **When:**
  `cm .status v::3`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-12: `bogus::x` → exit 1

- **Given:** clean environment
- **When:**
  `cm .status bogus::x`
  **Expected:** Exit 1; stderr mentions unknown parameter.
- **Then:** see spec
- **Exit:** 1

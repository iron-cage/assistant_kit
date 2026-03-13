# Test: `.status`

Integration test planning for the `.status` command. See [commands.md](../../commands.md) for specification.

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
| TC-095 | `.status` exits 0 always | P | 0 | F1=absent, F2=absent | [read_commands_test.rs] |
| TC-096 | `.status` with empty PATH → "not found", exits 0 | P | 0 | F3=empty PATH | [read_commands_test.rs] |
| TC-097 | `.status v::0` → exactly 3 bare lines | P | 0 | F1=0 | [read_commands_test.rs] |
| TC-098 | `.status v::1` → labeled Version/Sessions/Account lines | P | 0 | F1=1 | [read_commands_test.rs] |
| TC-100 | `.status format::json` → valid JSON with required keys | P | 0 | F2=json | [read_commands_test.rs] |
| TC-104 | `.status v::0` has fewer/equal lines than `.status v::1` | P | 0 | F1=0 vs 1 | [read_commands_test.rs] |
| TC-105 | `.status` HOME not set → account "unknown", no crash | P | 0 | F4=empty | [read_commands_test.rs] |
| TC-419 | `.status` with no preference → no "Preferred" line | P | 0 | F5=absent | [read_commands_test.rs] |
| TC-420 | `.status` with preference → shows "Preferred" line | P | 0 | F5=set | [read_commands_test.rs] |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| TC-451 | `.status format::xml` → exit 1 | N | 1 | F2=xml | new |
| TC-452 | `.status v::3` → exit 1, out of range | N | 1 | F1=3 | new |
| TC-453 | `.status bogus::x` → exit 1 | N | 1 | F6=present | new |

### Summary

- **Total:** 12 tests (9 positive, 3 negative)
- **Negative ratio:** 25.0% — below ≥40% threshold; covered by cross-cutting TC-242 to TC-244 and TC-245 in `read_commands_test.rs` which apply to `.status` among other commands
- **Existing cross-cutting negatives applying to `.status`:** TC-242 (`format::xml`), TC-243 (`format::JSON`), TC-244 (`format::`), TC-245 (`v::` duplication)
- **Combined negative count (command-specific + cross-cutting):** 7/16 = 43.8% ✅
- **TC range:** TC-095 to TC-453

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success (always — .status never errors) | TC-095 through TC-105, TC-419, TC-420 |
| 1 | Invalid arguments | TC-451 through TC-453 |
| 2 | Not applicable (.status always exits 0 for any valid state) | — |

### Degradation Semantics

`.status` exhibits unique behavior: it always exits 0 regardless of environment state.
Missing claude, missing HOME, or missing accounts produce informational "not found"/"unknown" output
rather than exit 2. This is by design (FR-01: status is read-only, never fails).

### Factor Coverage

| Factor | Positive Coverage | Negative Coverage |
|--------|-------------------|-------------------|
| F1 (v::) | TC-097 (v=0), TC-098 (v=1), TC-095 (absent) | TC-452 (v::3) |
| F2 (format) | TC-100 (json) | TC-451 (xml) |
| F3 (PATH) | TC-095 (found), TC-096 (empty) | — |
| F4 (HOME) | TC-095 (set), TC-105 (empty) | — |
| F5 (preference) | TC-419 (absent), TC-420 (set) | — |
| F6 (unknown params) | — | TC-453 |

---

## Test Case Details

### TC-095: `.status` exits 0 always

**Goal:** Basic invocation always succeeds.
**Setup:** None.
**Command:** `cm .status`
**Expected:** Exit 0.
**Verification:** exit code 0.
**Pass Criteria:** Exit 0.

---

### TC-096: Empty PATH → "not found", exits 0

**Goal:** Missing claude binary degrades gracefully.
**Setup:** `PATH=""`, `HOME=<tmp>`.
**Command:** `cm .status`
**Expected:** Exit 0; output contains "not found" or "unknown".
**Verification:** exit code 0; stdout contains "not found" or "unknown".
**Pass Criteria:** Exit 0; no crash.

---

### TC-097: `v::0` → exactly 3 bare lines

**Goal:** Minimum verbosity shows exactly 3 data points without labels, no preference line.
**Setup:** `HOME=<tmp>` with empty settings (no preference stored).
**Command:** `cm .status v::0`
**Expected:** Exactly 3 non-empty lines.
**Verification:** `text.lines().count() == 3`.
**Pass Criteria:** Exit 0; 3 lines.

---

### TC-098: `v::1` → labeled lines

**Goal:** Default verbosity shows `Version:`, `Sessions:`, `Account:` labels.
**Setup:** None.
**Command:** `cm .status v::1`
**Expected:** Output contains "Version:", "Sessions:", "Account:".
**Verification:** All three labels present.
**Pass Criteria:** Exit 0; all labels.

---

### TC-100: `format::json` → valid JSON

**Goal:** JSON format produces object with required keys.
**Setup:** None.
**Command:** `cm .status format::json`
**Expected:** JSON object with `version`, `processes`, `account` keys.
**Verification:** output contains `"version"`, `"processes"`, `"account"`.
**Pass Criteria:** Exit 0; required JSON fields present.

---

### TC-104: `v::0` has ≤ lines than `v::1`

**Goal:** Lower verbosity produces less or equal output.
**Setup:** None.
**Command:** Run both `v::0` and `v::1`.
**Expected:** Line count of `v::0` ≤ line count of `v::1`.
**Verification:** `n0 <= n1`.
**Pass Criteria:** Exit 0 for both; v::0 not longer than v::1.

---

### TC-105: HOME not set → "unknown" account, no crash

**Goal:** Missing HOME shows "unknown" for account, does not crash.
**Setup:** `HOME=""`.
**Command:** `cm .status`
**Expected:** Exit 0; stdout contains "unknown".
**Verification:** exit code 0; "unknown" present.
**Pass Criteria:** Exit 0; graceful degradation.

---

### TC-419: No preference → no "Preferred" line

**Goal:** Without stored `preferredVersionSpec`, status shows no "Preferred:" line.
**Setup:** `HOME=<tmp>`; `settings.json` has no `preferredVersionSpec`.
**Command:** `cm .status`
**Expected:** Output does not contain "Preferred".
**Verification:** stdout does not contain "Preferred".
**Pass Criteria:** Exit 0; no preference line.

---

### TC-420: With preference → shows "Preferred" line

**Goal:** Stored `preferredVersionSpec` appears as "Preferred:" line in status.
**Setup:** `HOME=<tmp>`; `settings.json` has `preferredVersionSpec = "stable"`.
**Command:** `cm .status`
**Expected:** Output contains "Preferred".
**Verification:** stdout contains "Preferred".
**Pass Criteria:** Exit 0; preference line present.

---

### TC-451: `format::xml` → exit 1

**Goal:** Unrecognized format values rejected at validation.
**Setup:** None.
**Command:** `cm .status format::xml`
**Expected:** Exit 1; stderr mentions unknown format.
**Verification:** exit code 1.
**Pass Criteria:** Exit 1.

---

### TC-452: `v::3` → exit 1, out of range

**Goal:** Out-of-range verbosity value rejected.
**Setup:** None.
**Command:** `cm .status v::3`
**Expected:** Exit 1.
**Verification:** exit code 1.
**Pass Criteria:** Exit 1.

---

### TC-453: `bogus::x` → exit 1

**Goal:** Unknown parameters rejected.
**Setup:** None.
**Command:** `cm .status bogus::x`
**Expected:** Exit 1; stderr mentions unknown parameter.
**Verification:** exit code 1.
**Pass Criteria:** Exit 1.

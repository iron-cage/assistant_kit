# Test: `.version.show`

Integration test planning for the `.version.show` command. See [commands.md](../../../../../docs/cli/commands.md) for specification.

## Test Factor Analysis

### Factor 1: `v::` / verbosity (Integer, optional, default 1)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default value 1, labeled output | Default behavior |
| 0 | Bare semver string only | Minimum output |
| 1 | `Version: X.Y.Z` labeled | Nominal |
| 2 | Extended detail (same as 1 if no extra data) | Maximum detail |
| 3 | Out-of-range integer | Invalid: exit 1 |

### Factor 2: `format::` (String, optional, default "text")

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default text output | Default behavior |
| `text` | Explicit text output | Explicit valid |
| `json` | `{"version":"X.Y.Z"}` | Alternate valid |
| `xml` | Unrecognized value | Invalid: exit 1 |

### Factor 3: claude binary availability (Environmental)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| available | Returns installed version | Happy path |
| unavailable | PATH empty or no claude | Failure: exit 2 |

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
| TC-108 | `.version.show v::0` → bare semver string | P | 0 | F1=0, F3=available | [read_commands_test.rs] |
| TC-109 | `.version.show v::1` → "Version: X.Y.Z" | P | 0 | F1=1, F3=available | [read_commands_test.rs] |
| TC-111 | `.version.show format::json` → `{"version":"..."}` | P | 0 | F2=json, F3=available | [read_commands_test.rs] |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| TC-107 | `.version.show` with no claude in PATH → exit 2 | N | 2 | F3=unavailable | [read_commands_test.rs] |
| TC-454 | `.version.show format::xml` → exit 1 | N | 1 | F2=xml | new |
| TC-455 | `.version.show v::3` → exit 1, out of range | N | 1 | F1=3 | new |
| TC-456 | `.version.show bogus::x` → exit 1 | N | 1 | F4=present | new |

### Summary

- **Total:** 7 tests (3 positive, 4 negative)
- **Negative ratio:** 57.1% ✅ (≥40%)
- **TC range:** TC-107 to TC-456

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success | TC-108, TC-109, TC-111 |
| 1 | Invalid arguments | TC-454, TC-455, TC-456 |
| 2 | Runtime error (claude not found) | TC-107 |

### Note on Network Conditionality

TC-108, TC-109, TC-111 are environment-conditional: if claude is not installed in the
test environment, the command exits 2 and the assertions are skipped. The tests use
`if out.status.code() == Some(0)` guards.

TC-107 is the inverse: it explicitly removes claude from PATH to force the exit 2 path.

---

## Test Case Details

### TC-107: No claude in PATH → exit 2

**Goal:** Missing claude binary causes exit 2.
**Setup:** `PATH=""`, `HOME=<tmp>`.
**Command:** `cm .version.show`
**Expected:** Exit 2.
**Verification:** exit code 2.
**Pass Criteria:** Exit 2.

---

### TC-108: `v::0` → bare semver string

**Goal:** Minimum verbosity shows only the version number.
**Setup:** claude installed (environment-conditional).
**Command:** `cm .version.show v::0`
**Expected:** Exit 0; stdout is a semver string only (digits and dots).
**Verification:** output consists only of digits and dots.
**Pass Criteria:** Exit 0; bare version string.
**Isolation:** Skipped if exit 2 (claude not installed).

---

### TC-109: `v::1` → "Version: X.Y.Z"

**Goal:** Default verbosity shows labeled output.
**Setup:** claude installed.
**Command:** `cm .version.show v::1`
**Expected:** Exit 0; output contains "Version:".
**Verification:** output contains "Version:".
**Pass Criteria:** Exit 0; "Version:" label present.
**Isolation:** Skipped if exit 2.

---

### TC-111: `format::json` → `{"version":"..."}`

**Goal:** JSON format output.
**Setup:** claude installed.
**Command:** `cm .version.show format::json`
**Expected:** Exit 0; output contains `"version"` JSON key.
**Verification:** output contains `"version"`.
**Pass Criteria:** Exit 0; JSON with version field.
**Isolation:** Skipped if exit 2.

---

### TC-454: `format::xml` → exit 1

**Goal:** Unrecognized format rejected.
**Setup:** None.
**Command:** `cm .version.show format::xml`
**Expected:** Exit 1.
**Verification:** exit code 1.
**Pass Criteria:** Exit 1.

---

### TC-455: `v::3` → exit 1

**Goal:** Out-of-range verbosity rejected.
**Setup:** None.
**Command:** `cm .version.show v::3`
**Expected:** Exit 1.
**Verification:** exit code 1.
**Pass Criteria:** Exit 1.

---

### TC-456: `bogus::x` → exit 1

**Goal:** Unknown parameter rejected.
**Setup:** None.
**Command:** `cm .version.show bogus::x`
**Expected:** Exit 1.
**Verification:** exit code 1.
**Pass Criteria:** Exit 1.

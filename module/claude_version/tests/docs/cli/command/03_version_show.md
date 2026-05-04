# Test: `.version.show`

Integration test planning for the `.version.show` command. See [commands.md](../../../../docs/cli/commands.md) for specification.

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
| IT-2 | `.version.show v::0` → bare semver string | P | 0 | F1=0, F3=available | [read_commands_test.rs] |
| IT-3 | `.version.show v::1` → "Version: X.Y.Z" | P | 0 | F1=1, F3=available | [read_commands_test.rs] |
| IT-4 | `.version.show format::json` → `{"version":"..."}` | P | 0 | F2=json, F3=available | [read_commands_test.rs] |

### Negative Tests

| TC | Description | P/N | Exit | Factors | Source |
|----|-------------|-----|------|---------|--------|
| IT-1 | `.version.show` with no claude in PATH → exit 2 | N | 2 | F3=unavailable | [read_commands_test.rs] |
| IT-5 | `.version.show format::xml` → exit 1 | N | 1 | F2=xml | new |
| IT-6 | `.version.show v::3` → exit 1, out of range | N | 1 | F1=3 | new |
| IT-7 | `.version.show bogus::x` → exit 1 | N | 1 | F4=present | new |
| IT-8 | Output goes to stdout only; stderr is empty | P | 0 | F3=available | new |

### Summary

- **Total:** 8 tests (4 positive, 4 negative)
- **Negative ratio:** 50.0% ✅ (≥40%)
- **TC range:** IT-1 to IT-8

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success | IT-2, IT-3, IT-4 |
| 1 | Invalid arguments | IT-5, IT-6, IT-7 |
| 2 | Runtime error (claude not found) | IT-1 |

### Note on Network Conditionality

IT-2, IT-3, IT-4 are environment-conditional: if claude is not installed in the
test environment, the command exits 2 and the assertions are skipped. The tests use
`if out.status.code() == Some(0)` guards.

IT-1 is the inverse: it explicitly removes claude from PATH to force the exit 2 path.

---

## Test Case Details

---

### IT-1: No claude in PATH → exit 2

- **Given:** `PATH=""`, `HOME=<tmp>`.
- **When:**
  `cm .version.show`
  **Expected:** Exit 2.
- **Then:** see spec
- **Exit:** 2

---

### IT-2: `v::0` → bare semver string

- **Given:** claude installed (environment-conditional).
- **When:**
  `cm .version.show v::0`
  **Expected:** Exit 0; stdout is a semver string only (digits and dots).
- **Then:** bare version string.
**Isolation:** Skipped if exit 2 (claude not installed)
- **Exit:** 0

---

### IT-3: `v::1` → "Version: X.Y.Z"

- **Given:** claude installed.
- **When:**
  `cm .version.show v::1`
  **Expected:** Exit 0; output contains "Version:".
- **Then:** "Version:" label present.
**Isolation:** Skipped if exit 2
- **Exit:** 0

---

### IT-4: `format::json` → `{"version":"..."}`

- **Given:** claude installed.
- **When:**
  `cm .version.show format::json`
  **Expected:** Exit 0; output contains `"version"` JSON key.
- **Then:** JSON with version field.
**Isolation:** Skipped if exit 2
- **Exit:** 0

---

### IT-5: `format::xml` → exit 1

- **Given:** clean environment
- **When:**
  `cm .version.show format::xml`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-6: `v::3` → exit 1

- **Given:** clean environment
- **When:**
  `cm .version.show v::3`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-7: `bogus::x` → exit 1

- **Given:** clean environment
- **When:**
  `cm .version.show bogus::x`
  **Expected:** Exit 1.
- **Then:** see spec
- **Exit:** 1

---

### IT-8: Output goes to stdout only; stderr is empty

- **Given:** clean environment with claude binary available
- **When:** `cm .version.show`
- **Then:** stdout is non-empty; stderr is empty
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

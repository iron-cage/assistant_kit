# Test: `count::`

Edge case coverage for the `count::` parameter. See [params.md](../../../../docs/cli/params.md) and [feature/001_version_management.md](../../../../docs/feature/001_version_management.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-426 | `count::3` → ≤3 version entries in output | Nominal |
| EC-1 | `count::0` → empty output, exit 0 | Boundary: minimum |
| TC-433 | `count::1 v::0` → exactly 1 bare line | Boundary: min useful |
| EC-2 | Absent `count::` → default 10 entries | Default Behavior |
| EC-3 | `count::100` → all available (capped by data) | Boundary: max |
| TC-439 | `count::0 format::json` → empty array `[]` | Boundary: empty JSON |
| EC-4 | `count::-1` → parse error → exit 1 | Invalid: negative |
| TC-447 | `v::abc` → exit 1 (type mismatch) | (companion: v:: type) |
| EC-5 | `count::abc` → exit 1 (type mismatch) | Invalid: type |
| EC-1 | `count::0` exits 0 (empty is not an error) | Empty vs Error |
| EC-2 | `count::` (empty) → exit 1 | Empty Value |
| EC-3 | `count::` only accepted by `.version.history` | Command Scope |
| EC-4 | Very large count (`count::9999`) → capped at data size | Boundary: very large |
| EC-6 | `count::18446744073709551615` (u64::MAX) → exit 1 | Overflow: above i64::MAX |
| EC-7 | `count::9223372036854775807` (i64::MAX) accepted → exit 0 | Boundary: i64::MAX |

## Test Coverage Summary

- Nominal: 1 test
- Boundary (0, 1, 100): 4 tests
- Default Behavior: 1 test
- Invalid (negative, type, empty): 3 tests
- Empty vs Error distinction: 1 test
- Command Scope: 1 test
- Very large boundary: 1 test
- Overflow (above i64::MAX): 1 test
- Boundary (i64::MAX accepted): 1 test

**Total:** 15 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

---

### EC-1: `count::0` → empty output, exit 0

- **Given:** Network available (or ignored if zero truncates before fetch).
- **When:** `cm .version.history count::0`
- **Then:** exit 0; stdout is empty.; valid-empty
- **Exit:** 0
- **Source:** [feature/001_version_management.md](../../../../docs/feature/001_version_management.md)

---

### EC-2: Absent → default 10

- **Given:** Network available.
- **When:** `cm .version.history`
- **Then:** ≤10 version lines.; default applied
- **Exit:** 0
- **Source:** [params.md — count:: default: 10](../../../../docs/cli/params.md)

---

### EC-3: `count::100` → all available

- **Given:** Network available; API has ~66 releases.
- **When:** `cm .version.history count::100 v::0`
- **Then:** exit 0; >10 lines (more than default).; data capped at available count
- **Exit:** 0
- **Source:** [feature/001_version_management.md](../../../../docs/feature/001_version_management.md)

---

### EC-4: `count::-1` → exit 1

- **Given:** clean environment
- **When:** `cm .version.history count::-1`
- **Then:** exit code 1 (adapter rejects negative as type error).
- **Exit:** 1
- **Source:** [params.md — count:: type: u64](../../../../docs/cli/params.md)
**Note:** Negative integers cannot be stored in u64; unilang adapter exits 1 on parse failure.

---

### EC-5: `count::abc` → exit 1

- **Given:** clean environment
- **When:** `cm .version.history count::abc`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-1: `count::0` is valid-empty (not an error)

- **Given:** Network available.
- **When:** `cm .version.history count::0`
- **Then:** exit 0; stdout empty; no stderr.; empty stdout; no error message
- **Exit:** 0
- **Source:** [feature/001_version_management.md](../../../../docs/feature/001_version_management.md)

---

### EC-2: `count::` (empty) → exit 1

- **Given:** clean environment
- **When:** `cm .version.history count::`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-3: `count::` only for `.version.history`

- **Given:** clean environment
- **When:** `cm .version.list count::3`
- **Then:** exit code 1; unknown parameter.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-4: Very large `count::9999` → capped at data size

- **Given:** Network available.
- **When:** `cm .version.history count::9999 v::0`
- **Then:** exit 0; ~66 lines (current API data size, not 9999).; line count equals API data size
- **Exit:** 0
- **Source:** [feature/001_version_management.md](../../../../docs/feature/001_version_management.md)

---

### EC-6: `count::18446744073709551615` (u64::MAX) → exit 1

- **Given:** clean environment
- **When:** `cm .version.history count::18446744073709551615`
- **Then:** exit code 1; error message mentions `count::`, NOT "fit in target type".; error is user-friendly (no internal unilang message)
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-7: `count::9223372036854775807` (i64::MAX) accepted → exit 0

- **Given:** Network available (command proceeds past validation; data caps at API size).
- **When:** `cm .version.history count::9223372036854775807 v::0`
- **Then:** exit code 0; output lines ≤ API data size.; (command succeeds; validation does not reject the boundary value)
- **Exit:** 0
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

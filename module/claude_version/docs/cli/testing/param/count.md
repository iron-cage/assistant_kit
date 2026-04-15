# Test: `count::`

Edge case coverage for the `count::` parameter. See [params.md](../../params.md) and [feature/001_version_management.md](../../../feature/001_version_management.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-426 | `count::3` → ≤3 version entries in output | Nominal |
| TC-427 | `count::0` → empty output, exit 0 | Boundary: minimum |
| TC-433 | `count::1 v::0` → exactly 1 bare line | Boundary: min useful |
| TC-435 | Absent `count::` → default 10 entries | Default Behavior |
| TC-436 | `count::100` → all available (capped by data) | Boundary: max |
| TC-439 | `count::0 format::json` → empty array `[]` | Boundary: empty JSON |
| TC-446 | `count::-1` → parse error → exit 1 | Invalid: negative |
| TC-447 | `v::abc` → exit 1 (type mismatch) | (companion: v:: type) |
| TC-448 | `count::abc` → exit 1 (type mismatch) | Invalid: type |
| EC-1 | `count::0` exits 0 (empty is not an error) | Empty vs Error |
| EC-2 | `count::` (empty) → exit 1 | Empty Value |
| EC-3 | `count::` only accepted by `.version.history` | Command Scope |
| EC-4 | Very large count (`count::9999`) → capped at data size | Boundary: very large |
| TC-487 | `count::18446744073709551615` (u64::MAX) → exit 1 | Overflow: above i64::MAX |
| TC-488 | `count::9223372036854775807` (i64::MAX) accepted → exit 0 | Boundary: i64::MAX |

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

---

### TC-427: `count::0` → empty output, exit 0

**Goal:** Zero count is valid — produces empty output, not an error.
**Setup:** Network available (or ignored if zero truncates before fetch).
**Command:** `cm .version.history count::0`
**Expected Output:** exit 0; stdout is empty.
**Verification:** exit code 0; stdout empty.
**Pass Criteria:** Exit 0; valid-empty.
**Source:** [feature/001_version_management.md](../../../feature/001_version_management.md)

---

### TC-435: Absent → default 10

**Goal:** Omitting `count::` returns at most 10 entries.
**Setup:** Network available.
**Command:** `cm .version.history`
**Expected Output:** ≤10 version lines.
**Verification:** line count ≤ 10.
**Pass Criteria:** Exit 0; default applied.
**Source:** [params.md — count:: default: 10](../../params.md)

---

### TC-436: `count::100` → all available

**Goal:** Count exceeding available releases returns all data, not an error.
**Setup:** Network available; API has ~66 releases.
**Command:** `cm .version.history count::100 v::0`
**Expected Output:** exit 0; >10 lines (more than default).
**Verification:** line count > 10 and ≤ 100.
**Pass Criteria:** Exit 0; data capped at available count.
**Source:** [feature/001_version_management.md](../../../feature/001_version_management.md)

---

### TC-446: `count::-1` → exit 1

**Goal:** Negative values are invalid for u64 integer parameter.
**Setup:** None.
**Command:** `cm .version.history count::-1`
**Expected Output:** exit code 1 (adapter rejects negative as type error).
**Verification:** exit code 1.
**Pass Criteria:** Exit 1.
**Source:** [params.md — count:: type: u64](../../params.md)
**Note:** Negative integers cannot be stored in u64; unilang adapter exits 1 on parse failure.

---

### TC-448: `count::abc` → exit 1

**Goal:** Non-integer string rejected for Integer-typed parameter.
**Setup:** None.
**Command:** `cm .version.history count::abc`
**Expected Output:** exit code 1.
**Verification:** exit code 1.
**Pass Criteria:** Exit 1.
**Source:** [feature/005_cli_design.md](../../../feature/005_cli_design.md)

---

### EC-1: `count::0` is valid-empty (not an error)

**Goal:** Distinguishes "zero count → empty" (exit 0) from "network error" (exit 2).
The system should never confuse "no results requested" with "failure".
**Setup:** Network available.
**Command:** `cm .version.history count::0`
**Expected Output:** exit 0; stdout empty; no stderr.
**Pass Criteria:** Exit 0; empty stdout; no error message.
**Source:** [feature/001_version_management.md](../../../feature/001_version_management.md)

---

### EC-2: `count::` (empty) → exit 1

**Goal:** Empty value for integer parameter is a usage error.
**Setup:** None.
**Command:** `cm .version.history count::`
**Expected Output:** exit code 1.
**Pass Criteria:** Exit 1.
**Source:** [feature/005_cli_design.md](../../../feature/005_cli_design.md)

---

### EC-3: `count::` only for `.version.history`

**Goal:** Other commands reject `count::` as unknown.
**Setup:** None.
**Command:** `cm .version.list count::3`
**Expected Output:** exit code 1; unknown parameter.
**Pass Criteria:** Exit 1.
**Source:** [feature/005_cli_design.md](../../../feature/005_cli_design.md)

---

### EC-4: Very large `count::9999` → capped at data size

**Goal:** Requesting more entries than available is not an error; returns all available.
Values this large are well below the `i64::MAX` adapter limit; see TC-487 for the overflow boundary.
**Setup:** Network available.
**Command:** `cm .version.history count::9999 v::0`
**Expected Output:** exit 0; ~66 lines (current API data size, not 9999).
**Pass Criteria:** Exit 0; line count equals API data size.
**Source:** [feature/001_version_management.md](../../../feature/001_version_management.md)

---

### TC-487: `count::18446744073709551615` (u64::MAX) → exit 1

**Goal:** Values above `i64::MAX` are rejected by the adapter with a user-friendly error.
Regression guard for issue-count-overflow: before the fix, u64 values above `i64::MAX`
were accepted by the adapter but triggered a cryptic "number too large to fit in target type"
error from unilang rather than a proper usage error.
**Setup:** None.
**Command:** `cm .version.history count::18446744073709551615`
**Expected Output:** exit code 1; error message mentions `count::`, NOT "fit in target type".
**Pass Criteria:** Exit 1; error is user-friendly (no internal unilang message).
**Source:** [feature/005_cli_design.md](../../../feature/005_cli_design.md)

---

### TC-488: `count::9223372036854775807` (i64::MAX) accepted → exit 0

**Goal:** The maximum accepted value is `i64::MAX` (9_223_372_036_854_775_807). Values at
exactly this boundary must NOT be rejected.
**Setup:** Network available (command proceeds past validation; data caps at API size).
**Command:** `cm .version.history count::9223372036854775807 v::0`
**Expected Output:** exit code 0; output lines ≤ API data size.
**Pass Criteria:** Exit 0 (command succeeds; validation does not reject the boundary value).
**Source:** [feature/005_cli_design.md](../../../feature/005_cli_design.md)

# Parameter :: `case_sensitive::`

Edge case tests for the `case_sensitive::` parameter. Tests validate boolean enforcement and search behavior impact.

**Source:** [params.md#parameter--2-case_sensitive](../../../../docs/cli/params.md#parameter--2-case_sensitive)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Value 0 uses case-insensitive matching | Behavior |
| EC-2 | Value 1 enables case-sensitive matching | Behavior |
| EC-3 | String "true" rejected | Type Validation |
| EC-4 | Omitted defaults to 0 (case-insensitive) | Default |
| EC-5 | case_sensitive::1 misses case-different matches | Behavior |
| EC-6 | case_sensitive::0 finds case-different matches | Behavior |

## Test Coverage Summary

- Behavior: 4 tests (EC-1, EC-2, EC-5, EC-6)
- Type Validation: 1 test (EC-3)
- Default: 1 test (EC-4)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases

---

### EC-1: Value 0 uses case-insensitive matching

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture session must contain the word "Error" with uppercase E)
- **When:** `clg .search query::error case_sensitive::0`
- **Then:** stdout contains match results including lines with "Error", "ERROR", and "error".; case-different matches appear in results
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-2: Value 1 enables case-sensitive matching

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture session must contain "Error" but not "error" in lowercase)
- **When:** `clg .search query::Error case_sensitive::1`
- **Then:** stdout contains matches for "Error" exactly; lowercase "error" entries are not returned.; only exact-case matches returned
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-3: String "true" rejected

- **Given:** clean environment
- **When:** `clg .search query::test case_sensitive::true`
- **Then:** stderr contains an error indicating `case_sensitive` must be 0 or 1.; error message `case_sensitive must be 0 or 1`
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-4: Omitted defaults to 0 (case-insensitive)

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture session must contain "Error" with uppercase E)
- **When:** `clg .search query::error`
- **Then:** stdout contains matches including "Error", "ERROR", and "error" — identical to EC-1 with explicit `case_sensitive::0`.; case-different matches appear (default case-insensitive behavior applied)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-5: case_sensitive::1 misses case-different matches

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture session must contain "ERROR" in all caps but not "error" in lowercase)
- **When:** `clg .search query::error case_sensitive::1`
- **Then:** stdout returns no matches (or fewer matches) because the lowercase query does not match the uppercase stored content.; uppercase-only stored content not returned when searching lowercase with case_sensitive::1
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-6: case_sensitive::0 finds case-different matches

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture session must contain "ERROR", "Error", and "error")
- **When:** `clg .search query::error case_sensitive::0`
- **Then:** stdout returns all three case variations: "ERROR", "Error", and "error".; all case variants of the query term returned in results
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

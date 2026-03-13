# Parameter :: `case_sensitive::`

Edge case tests for the `case_sensitive::` parameter. Tests validate boolean enforcement and search behavior impact.

**Source:** [params.md#parameter--2-case_sensitive](../../params.md#parameter--2-case_sensitive)

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

## Test Cases

### EC-1: Value 0 uses case-insensitive matching

**Goal:** Verify that `case_sensitive::0` causes `.search` to match content regardless of letter case.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture session must contain the word "Error" with uppercase E)
**Command:** `clg .search query::error case_sensitive::0`
**Expected Output:** stdout contains match results including lines with "Error", "ERROR", and "error".
**Verification:**
- Exit code is 0
- Results include matches where the query differs from the stored text only in case
**Pass Criteria:** exit 0 + case-different matches appear in results
**Source:** [params.md](../../params.md)

### EC-2: Value 1 enables case-sensitive matching

**Goal:** Verify that `case_sensitive::1` causes `.search` to only match content with exact letter case.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture session must contain "Error" but not "error" in lowercase)
**Command:** `clg .search query::Error case_sensitive::1`
**Expected Output:** stdout contains matches for "Error" exactly; lowercase "error" entries are not returned.
**Verification:**
- Exit code is 0
- Results include lines containing "Error" (exact case)
- Results do not include lines containing only "error" (different case)
**Pass Criteria:** exit 0 + only exact-case matches returned
**Source:** [params.md](../../params.md)

### EC-3: String "true" rejected

**Goal:** Verify that `case_sensitive::true` is rejected because only `0` and `1` are valid boolean values.
**Setup:** None
**Command:** `clg .search query::test case_sensitive::true`
**Expected Output:** stderr contains an error indicating `case_sensitive` must be 0 or 1.
**Verification:**
- Exit code is 1
- stderr contains a message like `case_sensitive must be 0 or 1`
**Pass Criteria:** exit 1 + error message `case_sensitive must be 0 or 1`
**Source:** [params.md](../../params.md)

### EC-4: Omitted defaults to 0 (case-insensitive)

**Goal:** Verify that omitting `case_sensitive::` causes `.search` to use case-insensitive matching (same as `case_sensitive::0`).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture session must contain "Error" with uppercase E)
**Command:** `clg .search query::error`
**Expected Output:** stdout contains matches including "Error", "ERROR", and "error" — identical to EC-1 with explicit `case_sensitive::0`.
**Verification:**
- Exit code is 0
- Output matches the result of `clg .search query::error case_sensitive::0`
**Pass Criteria:** exit 0 + case-different matches appear (default case-insensitive behavior applied)
**Source:** [params.md](../../params.md)

### EC-5: case_sensitive::1 misses case-different matches

**Goal:** Verify that with `case_sensitive::1`, a query in one case does not match stored content in a different case.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture session must contain "ERROR" in all caps but not "error" in lowercase)
**Command:** `clg .search query::error case_sensitive::1`
**Expected Output:** stdout returns no matches (or fewer matches) because the lowercase query does not match the uppercase stored content.
**Verification:**
- Exit code is 0
- Results do not contain lines with "ERROR" (all-caps) when searching for lowercase "error"
- Results are fewer than or equal to results from `case_sensitive::0`
**Pass Criteria:** exit 0 + uppercase-only stored content not returned when searching lowercase with case_sensitive::1
**Source:** [params.md](../../params.md)

### EC-6: case_sensitive::0 finds case-different matches

**Goal:** Verify that with `case_sensitive::0`, a query finds stored content regardless of case variation.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture session must contain "ERROR", "Error", and "error")
**Command:** `clg .search query::error case_sensitive::0`
**Expected Output:** stdout returns all three case variations: "ERROR", "Error", and "error".
**Verification:**
- Exit code is 0
- Results include entries containing "ERROR", "Error", and "error"
- Result count is equal to or greater than with `case_sensitive::1`
**Pass Criteria:** exit 0 + all case variants of the query term returned in results
**Source:** [params.md](../../params.md)

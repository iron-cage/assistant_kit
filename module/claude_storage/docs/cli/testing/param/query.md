# Parameter :: `query::`

Edge case tests for the `query::` parameter. Tests validate required enforcement, alias, and empty-value rejection.

**Source:** [params.md#parameter--11-query](../../params.md#parameter--11-query)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Required — missing query:: exits with 1 | Required Enforcement |
| EC-2 | Empty value rejected | Boundary Values |
| EC-3 | Single-word query accepted | Basic |
| EC-4 | Multi-word phrase query accepted (shell-quoted) | Basic |
| EC-5 | Alias q:: accepted same as query:: | Alias |
| EC-6 | Whitespace-only value rejected | Boundary Values |
| EC-7 | Query with special chars (e.g., ::) accepted | Basic |

## Test Coverage Summary

- Required Enforcement: 1 test (EC-1)
- Boundary Values: 2 tests (EC-2, EC-6)
- Basic: 3 tests (EC-3, EC-4, EC-7)
- Alias: 1 test (EC-5)

## Test Cases

### EC-1: Required — missing query:: exits with 1

**Goal:** Verify that running `.search` without `query::` exits with code 1 because the parameter is required.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .search`
**Expected Output:** Error indicating `query::` is required for `.search`.
**Verification:**
- Command exits with code 1
- Stderr contains an error message about the missing required `query::` parameter
**Pass Criteria:** exit 1 + error about missing required `query::` for `.search`
**Source:** [params.md](../../params.md)

---

### EC-2: Empty value rejected

**Goal:** Verify that an empty `query::` value is rejected with the exact error message.
**Setup:** None
**Command:** `clg .search query::`
**Expected Output:** `query must be non-empty`
**Verification:**
- Command exits with code 1
- Stderr contains the string `query must be non-empty`
**Pass Criteria:** exit 1 + error message `query must be non-empty`
**Source:** [params.md](../../params.md)

---

### EC-3: Single-word query accepted

**Goal:** Verify that a single-word search term is accepted and produces search results.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .search query::error`
**Expected Output:** Search results listing sessions and entries containing the word `error`.
**Verification:**
- Command exits with code 0
- Output contains session/entry matches for the term `error`
- No error about query format appears on stderr
**Pass Criteria:** exit 0 + search results returned for single-word query
**Source:** [params.md](../../params.md)

---

### EC-4: Multi-word phrase query accepted (shell-quoted)

**Goal:** Verify that a multi-word phrase passed as a shell-quoted argument is accepted and searched as a phrase.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .search query::"session management"`
**Expected Output:** Search results for entries containing the phrase `session management`.
**Verification:**
- Command exits with code 0
- No error about multi-word query format appears on stderr
- Results contain the phrase (or empty results if no matches — not an error)
**Pass Criteria:** exit 0 + multi-word phrase accepted and searched without format error
**Source:** [params.md](../../params.md)

---

### EC-5: Alias q:: accepted same as query::

**Goal:** Verify that the `q::` alias is functionally equivalent to `query::` for `.search`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .search q::error`
**Expected Output:** Same search results as `clg .search query::error`.
**Verification:**
- Command exits with code 0
- No error about unknown parameter `q::` appears on stderr
- Output matches what `query::error` would return
**Pass Criteria:** exit 0 + results identical to `query::error` results
**Source:** [params.md](../../params.md)

---

### EC-6: Whitespace-only value rejected

**Goal:** Verify that a whitespace-only string is rejected as an invalid query (treated as empty).
**Setup:** None
**Command:** `clg .search query::   ` (value is spaces only)
**Expected Output:** Error indicating query must be non-empty.
**Verification:**
- Command exits with code 1
- Stderr contains an error message indicating the query is invalid or empty
**Pass Criteria:** exit 1 + error about whitespace-only query value
**Source:** [params.md](../../params.md)

---

### EC-7: Query with special chars (e.g., ::) accepted

**Goal:** Verify that a query containing the `::` character sequence (the parameter delimiter) is accepted as literal search content.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .search query::param::value`
**Expected Output:** Search results for entries containing the literal string `param::value`.
**Verification:**
- Command exits with code 0
- No parsing error about the `::` inside the query value appears on stderr
- The search is performed for the literal string `param::value`
**Pass Criteria:** exit 0 + `::` within query value treated as literal content, not a second parameter delimiter
**Source:** [params.md](../../params.md)

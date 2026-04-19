# Parameter :: `entry_type::`

Edge case tests for the `entry_type::` parameter. Tests validate enum parsing, case-insensitivity, and default behavior.

**Source:** [params.md#parameter--4-entry_type](../../../../../docs/cli/params.md#parameter--4-entry_type) | [types.md#entrytype](../../../../../docs/cli/types.md#entrytype)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Value "user" accepted | Enum Values |
| EC-2 | Value "assistant" accepted | Enum Values |
| EC-3 | Value "all" accepted | Enum Values |
| EC-4 | Value "USER" accepted (case-insensitive) | Case Insensitivity |
| EC-5 | Invalid value "both" rejected with error | Error Handling |
| EC-6 | Invalid value "system" rejected with error | Error Handling |
| EC-7 | Omitted defaults to "all" | Default |

## Test Coverage Summary

- Enum Values: 3 tests (EC-1, EC-2, EC-3)
- Case Insensitivity: 1 test (EC-4)
- Error Handling: 2 tests (EC-5, EC-6)
- Default: 1 test (EC-7)

## Test Cases

### EC-1: Value "user" accepted

**Goal:** Verify that `entry_type::user` is accepted and filters search results to only user-authored entries.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .search query::error entry_type::user`
**Expected Output:** Search results containing only user-turn entries (not assistant responses) that match `error`.
**Verification:**
- Command exits with code 0
- No error about entry_type value appears on stderr
- All returned entries are of type `user` (none are assistant-authored)
**Pass Criteria:** exit 0 + only user-authored entries appear in results
**Source:** [params.md](../../../../../docs/cli/params.md)

---

### EC-2: Value "assistant" accepted

**Goal:** Verify that `entry_type::assistant` is accepted and filters search results to only assistant-authored entries.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .search query::error entry_type::assistant`
**Expected Output:** Search results containing only assistant-turn entries (not user messages) that match `error`.
**Verification:**
- Command exits with code 0
- No error about entry_type value appears on stderr
- All returned entries are of type `assistant` (none are user-authored)
**Pass Criteria:** exit 0 + only assistant-authored entries appear in results
**Source:** [params.md](../../../../../docs/cli/params.md)

---

### EC-3: Value "all" accepted

**Goal:** Verify that `entry_type::all` is accepted and applies no filter (returns both user and assistant entries).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .search query::error entry_type::all`
**Expected Output:** Search results containing both user and assistant entries matching `error`; identical to omitting `entry_type::`.
**Verification:**
- Command exits with code 0
- No error about entry_type value appears on stderr
- Result set is the union of `entry_type::user` and `entry_type::assistant` results
**Pass Criteria:** exit 0 + both user and assistant entries appear in results
**Source:** [params.md](../../../../../docs/cli/params.md)

---

### EC-4: Value "USER" accepted (case-insensitive)

**Goal:** Verify that uppercase input `USER` is accepted and treated identically to lowercase `user`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .search query::error entry_type::USER`
**Expected Output:** Same results as `entry_type::user` — case difference is normalized on parse.
**Verification:**
- Command exits with code 0
- No error about unrecognized value `USER` appears on stderr
- Result set is identical to `entry_type::user` result set
**Pass Criteria:** exit 0 + results match `entry_type::user` results exactly
**Source:** [params.md](../../../../../docs/cli/params.md)

---

### EC-5: Invalid value "both" rejected with error

**Goal:** Verify that the invalid value `both` produces the exact enum error message.
**Setup:** None
**Command:** `clg .search query::error entry_type::both`
**Expected Output:** `entry_type must be user|assistant|all, got both`
**Verification:**
- Command exits with code 1
- Stderr contains the string `entry_type must be user|assistant|all, got both`
**Pass Criteria:** exit 1 + error message `entry_type must be user|assistant|all, got both`
**Source:** [params.md](../../../../../docs/cli/params.md)

---

### EC-6: Invalid value "system" rejected with error

**Goal:** Verify that the invalid value `system` (a plausible but unsupported entry type) produces the exact enum error message.
**Setup:** None
**Command:** `clg .search query::error entry_type::system`
**Expected Output:** `entry_type must be user|assistant|all, got system`
**Verification:**
- Command exits with code 1
- Stderr contains the string `entry_type must be user|assistant|all, got system`
**Pass Criteria:** exit 1 + error message `entry_type must be user|assistant|all, got system`
**Source:** [params.md](../../../../../docs/cli/params.md)

---

### EC-7: Omitted defaults to "all"

**Goal:** Verify that omitting `entry_type::` defaults to `all` — both user and assistant entries are included in search results.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .search query::error`
**Expected Output:** Search results including both user and assistant entries matching `error`; same as `entry_type::all`.
**Verification:**
- Command exits with code 0
- Result set contains both user-authored and assistant-authored entries
- Result set is identical to `clg .search query::error entry_type::all`
**Pass Criteria:** exit 0 + results are equivalent to `entry_type::all` (no implicit filter applied)
**Source:** [params.md](../../../../../docs/cli/params.md)

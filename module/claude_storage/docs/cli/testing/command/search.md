# Command :: `.search`

Integration tests for the `.search` command. Tests verify query matching, scoping, case sensitivity, and entry type filtering.

**Source:** [commands.md#command--5-search](../../commands.md#command--5-search)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | query:: required — missing arg exits with 1 | Exit Codes |
| IT-2 | Case-insensitive match by default | Search Behavior |
| IT-3 | case_sensitive::1 enables exact case matching | Search Behavior |
| IT-4 | entry_type::user limits to user messages | Filtering |
| IT-5 | entry_type::assistant limits to assistant messages | Filtering |
| IT-6 | project:: restricts search to one project | Scoping |
| IT-7 | session:: restricts search to one session | Scoping |
| IT-8 | q alias works same as query | Alias |
| IT-9 | Phrase query with spaces (quoted) returns results | Search Behavior |
| IT-10 | Exit code 0 when results found | Exit Codes |

## Test Coverage Summary

- Exit Codes: 2 tests (IT-1, IT-10)
- Search Behavior: 3 tests (IT-2, IT-3, IT-9)
- Filtering: 2 tests (IT-4, IT-5)
- Scoping: 2 tests (IT-6, IT-7)
- Alias: 1 test (IT-8)

## Test Cases

### IT-1: query:: required — missing arg exits with 1

**Goal:** Verify `.search` without `query::` exits with code `1` and emits an argument error.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .search`
**Expected Output:** Error message on stderr indicating `query::` is required; no search results on stdout.
**Verification:**
- `$?` is `1`
- stderr contains an error message referencing missing `query` parameter
- stdout is empty
**Pass Criteria:** exit 1 + error message indicating `query::` is required

**Source:** [commands.md](../../commands.md)

---

### IT-2: Case-insensitive match by default

**Goal:** Verify search matches text regardless of case when `case_sensitive::` is not specified.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session with message containing the text `SessionManagement`)
**Command:** `clg .search query::sessionmanagement`
**Expected Output:** The session containing `SessionManagement` appears in results (matched case-insensitively).
**Verification:**
- stdout contains a result referencing the session with `SessionManagement`
- the match occurs despite case difference (`sessionmanagement` vs `SessionManagement`)
- stderr is empty
**Pass Criteria:** exit 0 + result found via case-insensitive match

**Source:** [commands.md](../../commands.md)

---

### IT-3: case_sensitive::1 enables exact case matching

**Goal:** Verify `case_sensitive::1` only matches text with the exact case provided in the query.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session with message containing `SessionManagement` but not `sessionmanagement`)
**Command:** `clg .search query::sessionmanagement case_sensitive::1`
**Expected Output:** No results (exact lowercase query does not match `SessionManagement`).
**Verification:**
- stdout contains no result entries (empty results or "no matches" indication)
- the session containing `SessionManagement` does not appear
- stderr is empty
**Pass Criteria:** exit 0 + no results for lowercase query when only mixed-case form exists

**Source:** [commands.md](../../commands.md)

---

### IT-4: entry_type::user limits to user messages

**Goal:** Verify `entry_type::user` restricts results to matches found only within user-authored entries.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session where `implement` appears in a user message and separately in an assistant message)
**Command:** `clg .search query::implement entry_type::user`
**Expected Output:** Only the user-message match is returned; the assistant-message match does not appear.
**Verification:**
- stdout contains the user message match result
- stdout does not contain a separate assistant message match for the same term
- entry type label or context confirms the match is from a user entry
**Pass Criteria:** exit 0 + only user-entry matches returned

**Source:** [commands.md](../../commands.md)

---

### IT-5: entry_type::assistant limits to assistant messages

**Goal:** Verify `entry_type::assistant` restricts results to matches found only within assistant-authored entries.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session where `implement` appears in both a user message and an assistant message)
**Command:** `clg .search query::implement entry_type::assistant`
**Expected Output:** Only the assistant-message match is returned; the user-message match does not appear.
**Verification:**
- stdout contains the assistant message match result
- stdout does not contain a separate user message match for the same term
- entry type label or context confirms the match is from an assistant entry
**Pass Criteria:** exit 0 + only assistant-entry matches returned

**Source:** [commands.md](../../commands.md)

---

### IT-6: project:: restricts search to one project

**Goal:** Verify `project::PROJECT` limits search scope to the named project only, excluding matches in other projects.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: projects `alpha` and `beta`, both contain the term `error`; search should only find `alpha` matches)
**Command:** `clg .search query::error project::alpha`
**Expected Output:** Only matches from project `alpha` appear; matches from `beta` are absent.
**Verification:**
- all result entries reference project `alpha`
- no result entries reference project `beta`
- stderr is empty
**Pass Criteria:** exit 0 + results scoped to project `alpha` only

**Source:** [commands.md](../../commands.md)

---

### IT-7: session:: restricts search to one session

**Goal:** Verify `session::SESSION` limits search scope to the named session only, excluding matches in other sessions within the same project.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha` with sessions `s1` and `s2`, both containing `refactor`; search scoped to `s1`)
**Command:** `clg .search query::refactor session::s1`
**Expected Output:** Only matches from session `s1` appear; matches from `s2` are absent.
**Verification:**
- all result entries reference session `s1`
- no result entries reference session `s2`
- stderr is empty
**Pass Criteria:** exit 0 + results scoped to session `s1` only

**Source:** [commands.md](../../commands.md)

---

### IT-8: q alias works same as query

**Goal:** Verify the `q` short alias for `query::` produces identical output to using `query::`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session containing the term `version_bump`)
**Command:** `clg .search q::version_bump`
**Expected Output:** Same results as `clg .search query::version_bump` — session containing `version_bump` found.
**Verification:**
- stdout contains the session match result for `version_bump`
- output is identical to running `clg .search query::version_bump` on the same fixture
- stderr is empty
**Pass Criteria:** exit 0 + identical results to `query::` form

**Source:** [commands.md](../../commands.md)

---

### IT-9: Phrase query with spaces (quoted) returns results

**Goal:** Verify a multi-word phrase query matches entries containing that exact phrase.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session with a user message containing the phrase `session management`)
**Command:** `clg .search query::"session management"`
**Expected Output:** The session containing `session management` appears in results.
**Verification:**
- stdout contains a result referencing the session with the `session management` phrase
- a session containing only `session` or only `management` separately does not appear (phrase match)
- stderr is empty
**Pass Criteria:** exit 0 + phrase-matched result returned

**Source:** [commands.md](../../commands.md)

---

### IT-10: Exit code 0 when results found

**Goal:** Verify `.search` exits with code `0` when the query returns at least one matching result.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: at least one session containing the term `error`)
**Command:** `clg .search query::error`
**Expected Output:** One or more result entries on stdout.
**Verification:**
- `$?` is `0`
- stdout contains at least one result entry
- stderr is empty
**Pass Criteria:** exit 0 + at least one result in output

**Source:** [commands.md](../../commands.md)

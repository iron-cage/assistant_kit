# Command :: `.search`

Integration tests for the `.search` command. Tests verify query matching, scoping, case sensitivity, and entry type filtering.

**Source:** [commands.md#command--5-search](../../../../docs/cli/commands.md#command--5-search)

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

---

### IT-1: query:: required — missing arg exits with 1

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .search`
- **Then:** Error message on stderr indicating `query::` is required; no search results on stdout.; error message indicating `query::` is required
- **Exit:** 1
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-2: Case-insensitive match by default

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session with message containing the text `SessionManagement`)
- **When:** `clg .search query::sessionmanagement`
- **Then:** The session containing `SessionManagement` appears in results (matched case-insensitively).; + result found via case-insensitive match
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-3: case_sensitive::1 enables exact case matching

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session with message containing `SessionManagement` but not `sessionmanagement`)
- **When:** `clg .search query::sessionmanagement case_sensitive::1`
- **Then:** No results (exact lowercase query does not match `SessionManagement`).; + no results for lowercase query when only mixed-case form exists
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-4: entry_type::user limits to user messages

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session where `implement` appears in a user message and separately in an assistant message)
- **When:** `clg .search query::implement entry_type::user`
- **Then:** Only the user-message match is returned; the assistant-message match does not appear.; + only user-entry matches returned
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-5: entry_type::assistant limits to assistant messages

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session where `implement` appears in both a user message and an assistant message)
- **When:** `clg .search query::implement entry_type::assistant`
- **Then:** Only the assistant-message match is returned; the user-message match does not appear.; + only assistant-entry matches returned
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-6: project:: restricts search to one project

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: projects `alpha` and `beta`, both contain the term `error`; search should only find `alpha` matches)
- **When:** `clg .search query::error project::alpha`
- **Then:** Only matches from project `alpha` appear; matches from `beta` are absent.; + results scoped to project `alpha` only
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-7: session:: restricts search to one session

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project `alpha` with sessions `s1` and `s2`, both containing `refactor`; search scoped to `s1`)
- **When:** `clg .search query::refactor session::s1`
- **Then:** Only matches from session `s1` appear; matches from `s2` are absent.; + results scoped to session `s1` only
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-8: q alias works same as query

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session containing the term `version_bump`)
- **When:** `clg .search q::version_bump`
- **Then:** Same results as `clg .search query::version_bump` — session containing `version_bump` found.; + identical results to `query::` form
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-9: Phrase query with spaces (quoted) returns results

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session with a user message containing the phrase `session management`)
- **When:** `clg .search query::"session management"`
- **Then:** The session containing `session management` appears in results.; + phrase-matched result returned
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-10: Exit code 0 when results found

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: at least one session containing the term `error`)
- **When:** `clg .search query::error`
- **Then:** One or more result entries on stdout.; + at least one result in output
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

# Command :: `.search`

Integration tests for the `.search` command. Tests verify query matching, scoping, case sensitivity, and entry type filtering.

**Source:** [command/05_search.md](../../../../docs/cli/command/05_search.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| INT-1 | query:: required — missing arg exits with 1 | Exit Codes |
| INT-2 | Case-insensitive match by default | Search Behavior |
| INT-3 | case_sensitive::1 enables exact case matching | Search Behavior |
| INT-4 | entry_type::user limits to user messages | Filtering |
| INT-5 | entry_type::assistant limits to assistant messages | Filtering |
| INT-6 | project:: restricts search to one project | Scoping |
| INT-7 | session:: restricts search to one session | Scoping |
| INT-8 | q alias works same as query | Alias |
| INT-9 | Phrase query with spaces (quoted) returns results | Search Behavior |
| INT-10 | Exit code 0 when results found | Exit Codes |

## Test Coverage Summary

- Exit Codes: 2 tests (INT-1, INT-10)
- Search Behavior: 3 tests (INT-2, INT-3, INT-9)
- Filtering: 2 tests (INT-4, INT-5)
- Scoping: 2 tests (INT-6, INT-7)
- Alias: 1 test (INT-8)

## Test Cases

---

### INT-1: query:: required — missing arg exits with 1

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .search
```

**Expected behavior:**
- Error message on stderr indicating `query::` is required; no search results on stdout
- Exit code: 1
- **Source:** [command/05_search.md](../../../../docs/cli/command/05_search.md)

---

### INT-2: Case-insensitive match by default

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .search query::sessionmanagement
```

**Expected behavior:**
- Fixture: session with message containing the text `SessionManagement`
- The session containing `SessionManagement` appears in results (matched case-insensitively)
- Exit code: 0
- **Source:** [command/05_search.md](../../../../docs/cli/command/05_search.md)

---

### INT-3: case_sensitive::1 enables exact case matching

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .search query::sessionmanagement case_sensitive::1
```

**Expected behavior:**
- Fixture: session with message containing `SessionManagement` but not `sessionmanagement`
- No results (exact lowercase query does not match `SessionManagement`)
- Exit code: 0
- **Source:** [command/05_search.md](../../../../docs/cli/command/05_search.md)

---

### INT-4: entry_type::user limits to user messages

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .search query::implement entry_type::user
```

**Expected behavior:**
- Fixture: session where `implement` appears in a user message and separately in an assistant message
- Only the user-message match is returned; the assistant-message match does not appear
- Exit code: 0
- **Source:** [command/05_search.md](../../../../docs/cli/command/05_search.md)

---

### INT-5: entry_type::assistant limits to assistant messages

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .search query::implement entry_type::assistant
```

**Expected behavior:**
- Fixture: session where `implement` appears in both a user message and an assistant message
- Only the assistant-message match is returned; the user-message match does not appear
- Exit code: 0
- **Source:** [command/05_search.md](../../../../docs/cli/command/05_search.md)

---

### INT-6: project:: restricts search to one project

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .search query::error project::alpha
```

**Expected behavior:**
- Fixture: projects `alpha` and `beta`, both contain the term `error`
- Only matches from project `alpha` appear; matches from `beta` are absent
- Exit code: 0
- **Source:** [command/05_search.md](../../../../docs/cli/command/05_search.md)

---

### INT-7: session:: restricts search to one session

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .search query::refactor session::s1
```

**Expected behavior:**
- Fixture: project `alpha` with sessions `s1` and `s2`, both containing `refactor`
- Only matches from session `s1` appear; matches from `s2` are absent
- Exit code: 0
- **Source:** [command/05_search.md](../../../../docs/cli/command/05_search.md)

---

### INT-8: q alias works same as query

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .search q::version_bump
```

**Expected behavior:**
- Fixture: session containing the term `version_bump`
- Same results as `clg .search query::version_bump` — session containing `version_bump` found
- Exit code: 0
- **Source:** [command/05_search.md](../../../../docs/cli/command/05_search.md)

---

### INT-9: Phrase query with spaces (quoted) returns results

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .search query::"session management"
```

**Expected behavior:**
- Fixture: session with a user message containing the phrase `session management`
- The session containing `session management` appears in results
- Exit code: 0
- **Source:** [command/05_search.md](../../../../docs/cli/command/05_search.md)

---

### INT-10: Exit code 0 when results found

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .search query::error
```

**Expected behavior:**
- Fixture: at least one session containing the term `error`
- One or more result entries on stdout
- Exit code: 0
- **Source:** [command/05_search.md](../../../../docs/cli/command/05_search.md)

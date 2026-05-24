# Command :: `.search`

Integration tests for the `.search` command. Tests verify query matching, scoping, case sensitivity, and entry type filtering.

**Source:** [001_commands.md#command--5-search](../../../../docs/cli/001_commands.md#command--5-search)

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

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .search
```

**Expected behavior:**
- Error message on stderr indicating `query::` is required; no search results on stdout
- Exit code: 1
- **Source:** [001_commands.md](../../../../docs/cli/001_commands.md)

---

### IT-2: Case-insensitive match by default

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .search query::sessionmanagement
```

**Expected behavior:**
- Fixture: session with message containing the text `SessionManagement`
- The session containing `SessionManagement` appears in results (matched case-insensitively)
- Exit code: 0
- **Source:** [001_commands.md](../../../../docs/cli/001_commands.md)

---

### IT-3: case_sensitive::1 enables exact case matching

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .search query::sessionmanagement case_sensitive::1
```

**Expected behavior:**
- Fixture: session with message containing `SessionManagement` but not `sessionmanagement`
- No results (exact lowercase query does not match `SessionManagement`)
- Exit code: 0
- **Source:** [001_commands.md](../../../../docs/cli/001_commands.md)

---

### IT-4: entry_type::user limits to user messages

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .search query::implement entry_type::user
```

**Expected behavior:**
- Fixture: session where `implement` appears in a user message and separately in an assistant message
- Only the user-message match is returned; the assistant-message match does not appear
- Exit code: 0
- **Source:** [001_commands.md](../../../../docs/cli/001_commands.md)

---

### IT-5: entry_type::assistant limits to assistant messages

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .search query::implement entry_type::assistant
```

**Expected behavior:**
- Fixture: session where `implement` appears in both a user message and an assistant message
- Only the assistant-message match is returned; the user-message match does not appear
- Exit code: 0
- **Source:** [001_commands.md](../../../../docs/cli/001_commands.md)

---

### IT-6: project:: restricts search to one project

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .search query::error project::alpha
```

**Expected behavior:**
- Fixture: projects `alpha` and `beta`, both contain the term `error`
- Only matches from project `alpha` appear; matches from `beta` are absent
- Exit code: 0
- **Source:** [001_commands.md](../../../../docs/cli/001_commands.md)

---

### IT-7: session:: restricts search to one session

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .search query::refactor session::s1
```

**Expected behavior:**
- Fixture: project `alpha` with sessions `s1` and `s2`, both containing `refactor`
- Only matches from session `s1` appear; matches from `s2` are absent
- Exit code: 0
- **Source:** [001_commands.md](../../../../docs/cli/001_commands.md)

---

### IT-8: q alias works same as query

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .search q::version_bump
```

**Expected behavior:**
- Fixture: session containing the term `version_bump`
- Same results as `clg .search query::version_bump` — session containing `version_bump` found
- Exit code: 0
- **Source:** [001_commands.md](../../../../docs/cli/001_commands.md)

---

### IT-9: Phrase query with spaces (quoted) returns results

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .search query::"session management"
```

**Expected behavior:**
- Fixture: session with a user message containing the phrase `session management`
- The session containing `session management` appears in results
- Exit code: 0
- **Source:** [001_commands.md](../../../../docs/cli/001_commands.md)

---

### IT-10: Exit code 0 when results found

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .search query::error
```

**Expected behavior:**
- Fixture: at least one session containing the term `error`
- One or more result entries on stdout
- Exit code: 0
- **Source:** [001_commands.md](../../../../docs/cli/001_commands.md)

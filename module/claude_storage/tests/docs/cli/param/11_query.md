# Parameter :: `query::`

Edge case tests for the `query::` parameter. Tests validate required enforcement, alias, and empty-value rejection.

**Source:** [param/11_query.md](../../../../docs/cli/param/11_query.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Required — missing query:: exits with 1 | Required Enforcement |
| EC-2 | Empty value rejected | Boundary Values |
| EC-3 | Single-word query accepted | Behavioral Divergence |
| EC-4 | Multi-word phrase query accepted (shell-quoted) | Behavioral Divergence |
| EC-5 | Alias q:: accepted same as query:: | Alias |
| EC-6 | Whitespace-only value rejected | Boundary Values |
| EC-7 | Query with special chars (e.g., ::) accepted | Basic |

## Test Coverage Summary

- Required Enforcement: 1 test (EC-1)
- Boundary Values: 2 tests (EC-2, EC-6)
- Behavioral Divergence: 2 tests (EC-3, EC-4)
- Basic: 1 test (EC-7)
- Alias: 1 test (EC-5)

**Total:** 7 edge cases

**Behavioral Divergence Pair:** EC-3 (single-word query, matches word) ↔ EC-4 (multi-word phrase, matches phrase)

## Test Cases

---

### EC-1: Required — missing query:: exits with 1

- **Commands:** `.search`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .search`
- **Then:** Error indicating `query::` is required for `.search`.; error about missing required `query::` for `.search`
- **Exit:** 1
- **Source:** [param/11_query.md](../../../../docs/cli/param/11_query.md)

---

### EC-2: Empty value rejected

- **Commands:** `.search`
- **Given:** clean environment
- **When:** `clg .search query::`
- **Then:** `query must be non-empty`; + error message `query must be non-empty`
- **Exit:** 1
- **Source:** [param/11_query.md](../../../../docs/cli/param/11_query.md)

---

### EC-3: Single-word query accepted

- **Commands:** `.search`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .search query::error`
- **Then:** Search results listing sessions and entries containing the word `error`.; + search results returned for single-word query
- **Exit:** 0
- **Source:** [param/11_query.md](../../../../docs/cli/param/11_query.md)

---

### EC-4: Multi-word phrase query accepted (shell-quoted)

- **Commands:** `.search`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .search query::"session management"`
- **Then:** Search results for entries containing the phrase `session management`.; + multi-word phrase accepted and searched without format error
- **Exit:** 0
- **Source:** [param/11_query.md](../../../../docs/cli/param/11_query.md)

---

### EC-5: Alias q:: accepted same as query::

- **Commands:** `.search`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .search q::error`
- **Then:** Same search results as `clg .search query::error`.; + results identical to `query::error` results
- **Exit:** 0
- **Source:** [param/11_query.md](../../../../docs/cli/param/11_query.md)

---

### EC-6: Whitespace-only value rejected

- **Commands:** `.search`
- **Given:** clean environment
- **When:** `clg .search query::   ` (value is spaces only)
- **Then:** Error indicating query must be non-empty.; + error about whitespace-only query value
- **Exit:** 1
- **Source:** [param/11_query.md](../../../../docs/cli/param/11_query.md)

---

### EC-7: Query with special chars (e.g., ::) accepted

- **Commands:** `.search`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .search query::param::value`
- **Then:** Search results for entries containing the literal string `param::value`.; + `::` within query value treated as literal content, not a second parameter delimiter
- **Exit:** 0
- **Source:** [param/11_query.md](../../../../docs/cli/param/11_query.md)

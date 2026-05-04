# Parameter :: `session::`

Edge case tests for the `session::` parameter (filter, not `session_id::`). Tests validate substring matching and auto-enable behavior.

**Source:** [params.md#parameter--13-session](../../../../docs/cli/params.md#parameter--13-session) | [types.md#sessionfilter](../../../../docs/cli/types.md#sessionfilter)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Partial match at start of session ID | Matching |
| EC-2 | Partial match in middle of session ID | Matching |
| EC-3 | Case-insensitive match | Matching |
| EC-4 | No match returns empty results | Matching |
| EC-5 | Empty value rejected | Boundary Values |
| EC-6 | Auto-enables sessions::1 in .list | Auto-Enable |
| EC-7 | session:: in .count restricts to matching session | Scoping |

## Test Coverage Summary

- Matching: 4 tests (EC-1, EC-2, EC-3, EC-4)
- Boundary Values: 1 test (EC-5)
- Auto-Enable: 1 test (EC-6)
- Scoping: 1 test (EC-7)

**Total:** 7 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases

---

### EC-1: Partial match at start of session ID

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list session::default`
- **Then:** Sessions whose ID starts with `default` (e.g., `-default_topic`) appear in results.; sessions starting with `default` in their ID are returned
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-2: Partial match in middle of session ID

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list session::topic`
- **Then:** Sessions containing `topic` anywhere in their ID (e.g., `-default_topic`) appear in results.; + sessions containing `topic` in their ID are returned
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-3: Case-insensitive match

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list session::DEFAULT`
- **Then:** Same sessions returned as `session::default` — case difference is ignored.; + same results as lowercase equivalent filter
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-4: No match returns empty results

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list session::zzznomatch999`
- **Then:** Empty session list or "no sessions found" message; no error exit code.; + empty result set (no error for non-matching filter)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-5: Empty value rejected

- **Given:** clean environment
- **When:** `clg .list session::`
- **Then:** Error about empty session filter value (e.g., `session filter must be non-empty`).; + error message about empty session filter
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-6: Auto-enables sessions::1 in .list

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list session::default`
- **Then:** Project list with session rows shown under each project; only sessions matching `default` are shown.; + session display auto-enabled and filtered by `default` substring
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-7: session:: in .count restricts to matching session

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .count target::entries session::default`
- **Then:** Entry count only for sessions matching `default`; not a total across all sessions.; + count scoped to matching sessions only
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

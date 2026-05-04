# Parameter :: `session_id::`

Edge case tests for the `session_id::` parameter (direct identifier, not filter). Tests validate required enforcement and not-found handling.

**Source:** [params.md#parameter--14-session_id](../../../../docs/cli/params.md#parameter--14-session_id) | [types.md#sessionid](../../../../docs/cli/types.md#sessionid)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Named session ID (e.g., -default_topic) accepted | Format |
| EC-2 | UUID session ID accepted | Format |
| EC-3 | Empty value rejected | Boundary Values |
| EC-4 | Unknown session ID exits with error | Error Handling |
| EC-5 | Required in .export — missing exits with 1 | Required Enforcement |
| EC-6 | Optional in .show — absent shows project | Optional Behavior |
| EC-7 | Whitespace-only value rejected | Boundary Values |

## Test Coverage Summary

- Format: 2 tests (EC-1, EC-2)
- Boundary Values: 2 tests (EC-3, EC-7)
- Error Handling: 1 test (EC-4)
- Required Enforcement: 1 test (EC-5)
- Optional Behavior: 1 test (EC-6)

**Total:** 7 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases

---

### EC-1: Named session ID (e.g., -default_topic) accepted

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show session_id::-default_topic`
- **Then:** Session content for the session stored as `-default_topic.jsonl` in the current project.; content from `-default_topic` session displayed
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-2: UUID session ID accepted

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show session_id::8d795a1c-c81d-4010-8d29-b4e678272419`
- **Then:** Session content for the session stored as `8d795a1c-c81d-4010-8d29-b4e678272419.jsonl`.; + content from UUID session displayed
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-3: Empty value rejected

- **Given:** clean environment
- **When:** `clg .show session_id::`
- **Then:** Error about empty session ID value (e.g., `session_id must be non-empty`).; + error about empty session_id value
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-4: Unknown session ID exits with error

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show session_id::-nonexistent-session-zzz`
- **Then:** `session not found: -nonexistent-session-zzz`; + error message `session not found: -nonexistent-session-zzz`
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-5: Required in .export — missing exits with 1

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .export output::/tmp/out.md`
- **Then:** Error indicating `session_id::` is required for `.export`.; + error about missing `session_id::` for `.export`
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-6: Optional in .show — absent shows project

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show` (run from a directory with a known project in the fixture)
- **Then:** Project-level view (list of sessions, project metadata) rather than a single session's content.; + project view shown (not a single-session view, not an error)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-7: Whitespace-only value rejected

- **Given:** clean environment
- **When:** `clg .show session_id::   ` (value is spaces only)
- **Then:** Error about invalid or empty session ID value.; + error about whitespace-only session_id value
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)
